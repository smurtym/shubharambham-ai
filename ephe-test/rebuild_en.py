#!/usr/bin/env python3
"""Rebuild the English (`en`) locale block of ephe/cities.csv from the Telugu
(`te`) rows (canonical_name is English in te rows), then merge the new cities
from ephe-test/src/cities2.csv. Telugu rows are preserved unchanged.

India `en` rows get individual English state names (te only groups them into
Telangana / Andhra Pradesh / "Other Indian States"); assignment is by city.
International `en` rows are a translation of the te region1/region2 labels,
keeping te's region1_order / region2_order.
"""
import csv
import os
from quadkey import encode_city_id

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CITIES = os.path.join(ROOT, "ephe", "cities.csv")
NEW = os.path.join(ROOT, "ephe-test", "src", "cities2.csv")

FIELDS = ["city_id", "canonical_name", "timezone", "lang", "city_name",
          "region1", "region2", "region1_order", "region2_order"]

# --- India: canonical city name -> real English state (for te India cities) ---
INDIA_STATE = {
    # Telangana (te group 1)
    "Adilabad": "Telangana", "Karimnagar": "Telangana", "Khammam": "Telangana",
    "Nalgonda": "Telangana", "Nizamabad": "Telangana", "Bhadrachalam": "Telangana",
    "Mancherial": "Telangana", "Mahabubnagar": "Telangana", "Warangal": "Telangana",
    "Hyderabad": "Telangana",
    # Andhra Pradesh (te group 2)
    "Anantapur": "Andhra Pradesh", "Annavaram": "Andhra Pradesh",
    "Amalapuram": "Andhra Pradesh", "Ichchapuram": "Andhra Pradesh",
    "Eluru": "Andhra Pradesh", "Ongole": "Andhra Pradesh", "Kadapa": "Andhra Pradesh",
    "Kurnool": "Andhra Pradesh", "Kakinada": "Andhra Pradesh", "Guntur": "Andhra Pradesh",
    "Chittoor": "Andhra Pradesh", "Tirupati": "Andhra Pradesh", "Nandyal": "Andhra Pradesh",
    "Nellore": "Andhra Pradesh", "Machilipatnam": "Andhra Pradesh",
    "Rajahmundry": "Andhra Pradesh", "Vijayawada": "Andhra Pradesh",
    "Visakhapatnam": "Andhra Pradesh", "Srikakulam": "Andhra Pradesh",
    "Hindupur": "Andhra Pradesh",
    # "Other Indian States" (te group 3) -> real states
    "Ahmedabad": "Gujarat", "Ujjain": "Madhya Pradesh", "Kolkata": "West Bengal",
    "Kochi": "Kerala", "Coimbatore": "Tamil Nadu", "Gaya": "Bihar",
    "Guwahati": "Assam", "Chandigarh": "Chandigarh", "Chennai": "Tamil Nadu",
    "Jaipur": "Rajasthan", "Delhi": "Delhi", "Thiruvananthapuram": "Kerala",
    "Patna": "Bihar", "Pune": "Maharashtra", "Berhampur": "Odisha",
    "Ballari": "Karnataka", "Bengaluru": "Karnataka", "Bhubaneswar": "Odisha",
    "Bhopal": "Madhya Pradesh", "Mangaluru": "Karnataka", "Madurai": "Tamil Nadu",
    "Mumbai": "Maharashtra", "Mysuru": "Karnataka", "Raipur": "Chhattisgarh",
    "Lucknow": "Uttar Pradesh", "Varanasi": "Uttar Pradesh",
}

# --- International: te region1 -> en region1 ---
REGION1_EN = {
    # Asia
    "ఇజ్రాయిల్": "Israel", "ఖతార్": "Qatar", "చైనా": "China", "జపాన్": "Japan",
    "టిబెట్": "Tibet", "తైవాన్": "Taiwan", "థాయ్‌లాండ్": "Thailand",
    "దక్షిణకొరియా": "South Korea", "నేపాల్": "Nepal", "బంగ్లాదేశ్": "Bangladesh",
    "బహ్రెయిన్": "Bahrain", "మయన్మార్": "Myanmar", "మలేషియా": "Malaysia",
    "శ్రీలంక": "Sri Lanka", "సౌదీ అరేబియా": "Saudi Arabia", "సౌదీఅరేబియా": "Saudi Arabia",
    "యునైటెడ్ అరబ్ ఎమిరేట్స్": "United Arab Emirates", "సింగపూర్": "Singapore",
    "హాంకాంగ్": "Hong Kong",
    # North America (US states + Canada + Mexico)
    "అరిజోనా": "Arizona", "ఇండియానా": "Indiana", "ఇలెనోయి": "Illinois",
    "ఓక్లహోమా": "Oklahoma", "ఓరిగాన్": "Oregon", "ఓహైయో": "Ohio",
    "కనెటికట్": "Connecticut", "కాలిఫోర్నియా": "California", "కెంటకీ": "Kentucky",
    "కెనడా": "Canada", "కొలరాడో": "Colorado", "జార్జియా": "Georgia",
    "టెక్సాస్": "Texas", "టెన్నెస్సీ": "Tennessee", "డెలవేర్": "Delaware",
    "నార్త్ కరోలినా": "North Carolina", "నెబ్రాస్కా": "Nebraska",
    "న్యూజెర్సి": "New Jersey", "పెన్సెల్‌వేనియా": "Pennsylvania", "ఫ్లోరిడా": "Florida",
    "మాసాచుసెట్స్": "Massachusetts", "మినెసోటా": "Minnesota", "మిషిగన్": "Michigan",
    "మిస్సోరి": "Missouri", "మెక్సికో": "Mexico", "మేరిలాండ్": "Maryland",
    "యుటా": "Utah", "వర్జీనియా": "Virginia", "వాషింగ్టన్": "Washington",
    "న్యూయార్క్": "New York", "వాషింగ్టన్ డిసి": "Washington DC",
    # Europe
    "ఆస్ట్రియా": "Austria", "ఐర్లాండ్": "Ireland", "జర్మని": "Germany",
    "డెన్మార్క్": "Denmark", "నార్వే": "Norway", "నెదర్లాండ్": "Netherlands",
    "నెదర్లాండ్స్": "Netherlands", "పోర్చుగల్": "Portugal", "పోలాండ్": "Poland",
    "ఫిన్‌లాండ్": "Finland", "ఫ్రాన్స్": "France", "యునైటెడ్ కింగ్‌డమ్": "United Kingdom",
    "రష్యా": "Russia", "స్పెయిన్": "Spain", "స్వీడన్": "Sweden",
    # Australia / NZ
    "ఆస్ట్రేలియా": "Australia", "న్యూజిలాండ్": "New Zealand",
    # Africa
    "ఈజిప్ట్": "Egypt", "కెన్యా": "Kenya", "దక్షిణాఫ్రికా": "South Africa",
    "నైజీరియా": "Nigeria",
    # South America
    "అర్జంటినా": "Argentina", "చిలి": "Chile", "బ్రెజిల్": "Brazil",
}

# --- te region2 (country/region group) -> en region2 ---
REGION2_EN = {
    "భారతదేశం": "India",
    "ఇతర ఆసియా దేశాలు": "Other Asian Countries",
    "ఉత్తర అమెరికా": "North America",
    "యూరోప్": "Europe",
    "ఆస్ట్రేలియా/న్యూజిలాండ్": "Australia/New Zealand",
    "ఆఫ్రికా": "Africa",
    "దక్షిణ అమెరికా": "South America",
}


def ascii_name(s):
    return s.replace("–", "-").replace("—", "-")


# --- read existing file ---
with open(CITIES, encoding="utf-8") as f:
    lines = f.readlines()
idx = 0
header_comment = []
while lines[idx].rstrip("\n").split(",") != FIELDS:
    header_comment.append(lines[idx].rstrip("\n"))
    idx += 1
data_lines = [l.rstrip("\n") for l in lines[idx + 1:] if l.strip()]
rows = list(csv.DictReader(data_lines, fieldnames=FIELDS))
te_rows = [r for r in rows if r["lang"] == "te"]
assert len(te_rows) == 158, len(te_rows)

# --- build en rows from te rows ---
en_rows = []
for t in te_rows:
    canon = t["canonical_name"]
    if t["region2"] == "భారతదేశం":  # India
        region1 = INDIA_STATE[canon]
        region2 = "India"
    else:
        region1 = REGION1_EN[t["region1"]]
        region2 = REGION2_EN[t["region2"]]
    en_rows.append({
        "city_id": t["city_id"],
        "canonical_name": canon,
        "timezone": t["timezone"],
        "lang": "en",
        "city_name": canon,
        "region1": region1,
        "region2": region2,
        "region1_order": t["region1_order"],   # India re-numbered below
        "region2_order": t["region2_order"],
    })

# --- read & add new cities (all India, en only) ---
existing_ids = {r["city_id"] for r in rows}
new_rows = []
with open(NEW, encoding="utf-8") as f:
    for row in csv.DictReader(f, delimiter="\t"):
        name = ascii_name(row["City"].strip())
        state = row["State"].strip()
        cid = str(encode_city_id(float(row["Lat"]), float(row["Long"])))
        if cid in existing_ids:
            print(f"SKIP duplicate city_id {cid} {name} ({state})")
            continue
        existing_ids.add(cid)
        new_rows.append({
            "city_id": cid, "canonical_name": name, "timezone": "Asia/Kolkata",
            "lang": "en", "city_name": name, "region1": state, "region2": "India",
            "region1_order": "0", "region2_order": "1",
        })

# --- merge India en rows, renumber state order alphabetically ---
en_india = [r for r in en_rows if r["region2"] == "India"] + new_rows
en_world = [r for r in en_rows if r["region2"] != "India"]
states = sorted({r["region1"] for r in en_india}, key=str.lower)
order = {s: i + 1 for i, s in enumerate(states)}
for r in en_india:
    r["region1_order"] = str(order[r["region1"]])
    r["region2_order"] = "1"

# --- sort & assemble: te (unchanged) + en (India then world) ---
en_india.sort(key=lambda r: (int(r["region1_order"]), r["canonical_name"].lower()))
en_world.sort(key=lambda r: (int(r["region2_order"]), int(r["region1_order"]),
                             r["canonical_name"].lower()))
out_rows = te_rows + en_india + en_world

with open(CITIES, "w", encoding="utf-8", newline="") as f:
    for c in header_comment:
        f.write(c + "\n")
    f.write(",".join(FIELDS) + "\n")
    w = csv.DictWriter(f, fieldnames=FIELDS, lineterminator="\n")
    for r in out_rows:
        w.writerow(r)

print(f"te rows: {len(te_rows)}  en rows: {len(en_india) + len(en_world)} "
      f"(India {len(en_india)}, world {len(en_world)})  new: {len(new_rows)}")
print(f"India en states ({len(states)}):")
for s in states:
    n = sum(1 for r in en_india if r["region1"] == s)
    print(f"  {order[s]:2d}  {s:30s} {n}")
