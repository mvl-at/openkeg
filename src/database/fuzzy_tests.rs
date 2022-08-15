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
mod fuzzy_tests {
    use super::super::*;
    use regex::Regex;

    fn matches_fuzzy(search_term: &str, stored: &str) -> bool {
        Regex::new(fuzzy_regex(search_term.to_string()).as_str())
            .map(|r| r.is_match(stored))
            .expect("regex")
    }

    #[test]
    fn regex_umlauts() {
        assert_eq!(matches_fuzzy("Österreich", "Österreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Österreich"), true);
    }

    #[test]
    fn regex_case() {
        assert_eq!(matches_fuzzy("Osterreich", "osterreich"), true);
        assert_eq!(matches_fuzzy("OsTErrEich", "osteRreiCh"), true);
    }

    #[test]
    fn regex_special() {
        assert_eq!(matches_fuzzy("Oster.reich", "Osterreich"), true);
        assert_eq!(matches_fuzzy("Oster reich", "Osterreich"), true);
        assert_eq!(matches_fuzzy("Oster!reich", "Osterreich"), true);
        assert_eq!(matches_fuzzy("Ost?erreich", "Osterreich"), true);

        assert_eq!(matches_fuzzy("Osterreich", "Oster.reich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Oster reich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Oster!reich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost?erreich"), true);
    }

    #[test]
    fn regex_escape() {
        assert_eq!(matches_fuzzy("Oster.eich", "Osterreich"), false);
        assert_eq!(matches_fuzzy("Osterreich", "Ost,erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost.erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost-erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost-erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost;erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost:erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost_erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost<erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost>erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost+erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost\"erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost*erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost#erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost%erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost&erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost$erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost|erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost§erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost=erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost?erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost`erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost°erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost(erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost)erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost!erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost~erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost[erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost]erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost{erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost}erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost/erreich"), true);
        assert_eq!(matches_fuzzy("Osterreich", "Ost\\erreich"), true);
    }

    #[test]
    fn regex_numbers() {
        assert_eq!(matches_fuzzy("4 religiös", "4 Religiöse Aufzüge"), true);
        assert_eq!(
            matches_fuzzy("4religiöseAufzüge", "4--Religiöse Aufzüge"),
            true
        );
    }
}
