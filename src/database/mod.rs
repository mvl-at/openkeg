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

/// A module which contains generic functionality for the database.
/// The most important are client initialization, authentication, request and response types.
pub mod client;
/// Module which is responsible to provide fuzzy search.
/// This is implemented with regular expressions.
mod fuzzy;
/// Module which contains the database requests for score related services.
pub mod score;
/// Module which contains the database requests for statistic related services.
pub mod statistic;
