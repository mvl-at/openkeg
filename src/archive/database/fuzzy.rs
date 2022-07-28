// OpenKeg, the lightweight backend of the Musikverein Leopoldsdorf.
// Copyright (C) 2022  Richard Stöckl
//
// This program is free software; you can redistribute it and/or
// modify it under the terms of the GNU General Public License
// as published by the Free Software Foundation; either version 2
// of the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301, USA.

#[cfg(test)]
#[path = "fuzzy_tests.rs"]
mod fuzzy_tests;

/// Convert the search term into a fuzzy one.
/// The resulting regex may be very large and inefficient.
/// It is not recommended using this function when an intelligent string library which ignore diacritics is available.
///
/// # Arguments
///
/// * `term`: the term to convert
///
/// returns: String
pub fn fuzzy_regex(term: String) -> String {
    term.chars()
        .map(|c| {
            let next = ALPHABET_CLASSES.iter().filter(|cl| cl.contains(c)).next();

            let chars = next.map(|cl| format!("[{}]{}", cl, SPECIALS)).unwrap_or({
                if NUMBERS.contains(c) {
                    format!("{}{}", c, SPECIALS)
                } else {
                    "".to_string()
                }
            });
            chars
        })
        .collect::<Vec<String>>()
        .join("")
}

const ALPHABET_CLASSES: &'static [&'static str] = &[
    "aàáâãäåæAÀÁÂÃÄÅÆ",
    "bB",
    "cçćĉčCÇĆĈČ",
    "dďDÐĎ",
    "eèéêëēěEÈÉÊËĒĚ",
    "fF",
    "gĝGĜ",
    "hĥHĤ",
    "iìíîïIÌÍÎÏ",
    "jJ",
    "kK",
    "lL",
    "mM",
    "nñńňNÑŃŇ",
    "oòóôõöøOÒÓÔÕÖØ",
    "pP",
    "qQ",
    "rŕřRŔŘ",
    "sśŝşšSŚŜŞŠß",
    "tťTŤ",
    "uùúûüUÙÚÛÜ",
    "vV",
    "wŵWŴ",
    "xX",
    "yýŷYÝŶ",
    "zžZŹ",
];

// const SPECIAL_CHARACTERS: &str = "`°+\"*#%&$|§=?€<>,.-;:_()!~[]{}/\\ ";
const NUMBERS: &str = "0123456789";
const SPECIALS: &str = r#"[`°\+"'\^\*\#%&\$\|§=\?€<>,\.\-;:_\(\)!~\[\]\{\}/\\ ]*"#;
