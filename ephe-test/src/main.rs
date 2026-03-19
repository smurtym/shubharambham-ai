use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_double};

pub type Int32 = ::std::os::raw::c_int;
pub type CChar = ::std::os::raw::c_char;

// ---------------------------------------------------------------------------
// Swiss Ephemeris constants — values verified against vendor/swisseph/swephexp.h
// ---------------------------------------------------------------------------
const SE_SUN:            c_int = 0;
const SE_MOON:           c_int = 1;
const SE_MERCURY:        c_int = 2;
const SE_VENUS:          c_int = 3;
const SE_MARS:           c_int = 4;
const SE_JUPITER:        c_int = 5;
const SE_SATURN:         c_int = 6;
const SE_TRUE_NODE:      c_int = 11;
const SEFLG_SWIEPH:      c_int = 2;
const SEFLG_TRUEPOS:     c_int = 16;
const SEFLG_SPEED:       c_int = 256;
const SEFLG_SIDEREAL:    c_int = 65536;
const SE_SIDM_TRUE_CITRA: c_int = 27;
const SE_GREG_CAL:        c_int = 1;

// ---------------------------------------------------------------------------
// FFI — only the functions we need
// ---------------------------------------------------------------------------
extern "C" {
    fn swe_set_ephe_path(path: *const c_char);
    fn swe_set_sid_mode(sid_mode: c_int, t0: c_double, ayan_t0: c_double);
    fn swe_julday(year: c_int, month: c_int, day: c_int, hour: c_double, gregflag: c_int) -> c_double;
    fn swe_calc_ut(tjd_ut: c_double, ipl: c_int, iflag: c_int, xx: *mut c_double, serr: *mut c_char) -> c_int;
    fn swe_close();
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
const SIGNS: [&str; 12] = [
    "Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo",
    "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces",
];

const NAKSHATRAS: [&str; 27] = [
    "Ashwini", "Bharani", "Krittika", "Rohini", "Mrigashira", "Ardra",
    "Punarvasu", "Pushya", "Ashlesha", "Magha", "Purva Phalguni", "Uttara Phalguni",
    "Hasta", "Chitra", "Swati", "Vishakha", "Anuradha", "Jyeshtha",
    "Mula", "Purva Ashadha", "Uttara Ashadha", "Shravana", "Dhanishtha",
    "Shatabhisha", "Purva Bhadrapada", "Uttara Bhadrapada", "Revati",
];

fn normalize(lon: f64) -> f64 {
    ((lon % 360.0) + 360.0) % 360.0
}

fn decompose(lon: f64) -> (usize, u8, u8, u8, usize, u8) {
    let lon = normalize(lon);
    let sign_idx  = (lon / 30.0) as usize;           // 0-based
    let deg       = (lon % 30.0) as u8;
    let min_frac  = (lon % 30.0 - deg as f64) * 60.0;
    let min       = min_frac as u8;
    let sec       = ((min_frac - min as f64) * 60.0) as u8;
    let nak_idx   = (lon / (360.0 / 27.0)) as usize; // 0-based
    let pada      = ((lon % (360.0 / 27.0)) / (360.0 / 108.0)) as u8 + 1;
    (sign_idx, deg, min, sec, nak_idx, pada)
}

fn calc(jd: f64, body: c_int, iflag: c_int) -> Result<f64, String> {
    let mut xx   = [0.0_f64; 6];
    let mut serr = [0_u8 as c_char; 256];
    let (ret, msg) = unsafe {
        let r = swe_calc_ut(jd, body, iflag, xx.as_mut_ptr(), serr.as_mut_ptr());
        let m = CStr::from_ptr(serr.as_mut_ptr()).to_string_lossy().into_owned();
        (r, m)
    };
    if ret < 0 || (ret & SEFLG_SWIEPH) == 0 || (ret & SEFLG_SIDEREAL) == 0 {
        // serr is cleared by a second internal swe_calc call inside swe_calc_ut
        // (delta-T retry). Use calc_ut (single swe_calc call) to recover the message.
        let mut serr2 = [0_u8 as c_char; 256];
        let msg = if msg.is_empty() {
            unsafe {
                swe_calc_ut(jd, body, SEFLG_SWIEPH | SEFLG_TRUEPOS | SEFLG_SIDEREAL,
                    xx.as_mut_ptr(), serr2.as_mut_ptr());
                CStr::from_ptr(serr2.as_mut_ptr()).to_string_lossy().into_owned()
            }
        } else {
            msg
        };
        eprintln!("swe warning/error: {msg}");
        return Err(msg);
    }
    Ok(normalize(xx[0]))
}

pub fn set_ephe_path() {
    let path = std::ffi::CString::new("data").unwrap();
    print!("Setting ephemeris path to: {:?}", path);
    unsafe {

        swe_set_ephe_path(path.as_ptr());
        // Setting ayanamsa
        // swe_set_sid_mode(SE_SIDM_TRUE_CITRA, 0.0, 0.0);
    }
}

pub fn calc_ut(tjd_ut: f64, ipl: Int32, iflag: Int32) -> f64 {
    let mut xx: [f64; 6] = [0.0; 6];
    let mut serr: [CChar; 256] = [0; 256];
    let err;
    unsafe {
        swe_calc_ut(tjd_ut, ipl, iflag, xx.as_mut_ptr(), serr.as_mut_ptr());
        err = std::ffi::CStr::from_ptr(serr.as_mut_ptr()).to_str().unwrap();
    }
    // convert serr only if it's not empty
    if err != "" {
        println!("Error: {}", err);
    }

    xx[0]
}

pub fn get_sun_ephemeris(tjd_ut: f64) -> f64 {
    let ipl = SE_SUN; // Sun's planet number in Swiss Ephemeris
    let iflag = SEFLG_SWIEPH | SEFLG_TRUEPOS | SEFLG_SIDEREAL ;//| SEFLG_NONUT; 
    set_ephe_path();
    calc_ut(tjd_ut, ipl, iflag)
}


fn print_row(name: &str, lon: f64) {
    let (sign_idx, deg, min, sec, nak_idx, pada) = decompose(lon);
    println!(
        "{:<8} {:>10.4}°  {:<12} {:02}°{:02}'{:02}\"  {:<21} P{}",
        name, lon, SIGNS[sign_idx], deg, min, sec, NAKSHATRAS[nak_idx], pada
    );
}

fn main1() {
    let jd = 2461118.145833;
     println!("Sun ephe:meris (via calc_ut): {:.4}°", get_sun_ephemeris(jd));

    //  main1();
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
fn main() {
    // 2026-03-18 21:00 IST = 2026-03-18 15:30 UTC
    let (year, month, day, hour_utc) = (2026_i32, 3_i32, 18_i32, 15.5_f64);

    let jd = 2461118.145833;
     println!("Sun ephe:meris (via calc_ut): {:.4}°", get_sun_ephemeris(jd));

    let jd = unsafe {
        swe_set_ephe_path(b"../ephe/\0".as_ptr() as *const c_char);
        // swe_set_sid_mode(SE_SIDM_TRUE_CITRA, 0.0, 0.0);
        swe_julday(year, month, day, hour_utc, SE_GREG_CAL)
    };

    println!("Date : 2026-03-18 21:00 IST  (15:30 UTC)");
    println!("JD   : {jd:.6}");
    println!("Ayanamsa: True Chitrapaksha (SE_SIDM_TRUE_CITRA)\n");
    println!("{:<8} {:>10}   {:<12} {:>10}  {:<21} {}", "Planet", "Longitude", "Sign", "DMS", "Nakshatra", "Pada");
    println!("{}", "-".repeat(78));

    let iflag = SEFLG_SIDEREAL | SEFLG_TRUEPOS | SEFLG_SWIEPH;
//println!("Sun ephe:meris (via calc_ut): {:.4}°", get_sun_ephemeris(jd));
    let bodies: &[(&str, c_int)] = &[
        ("Sun",     SE_SUN),
        ("Moon",    SE_MOON),
        ("Mercury", SE_MERCURY),
        ("Venus",   SE_VENUS),
        ("Mars",    SE_MARS),
        ("Jupiter", SE_JUPITER),
        ("Saturn",  SE_SATURN),
        ("Rahu",    SE_TRUE_NODE),
    ];

    let mut rahu_lon = 0.0_f64;

    for (name, body) in bodies {
        match calc(jd, *body, iflag) {
            Ok(lon) => {
                if *name == "Rahu" { rahu_lon = lon; }
                print_row(name, lon);
            }
            Err(e) => eprintln!("{name}: ERROR — {e}"),
        }
    }

    // Ketu = Rahu + 180°
    let ketu_lon = normalize(rahu_lon + 180.0);
    print_row("Ketu", ketu_lon);

    unsafe { swe_close(); }

    //let jd = 2461118.145833;
     println!("Sun ephe:meris (via calc_ut): {:.4}°", get_sun_ephemeris(jd));

    
}
