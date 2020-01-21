// Chariot: An open source reimplementation of Age of Empires (1997)
// Copyright (c) 2016 Kevin Fuller
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//

use crate::error::Result;

use identifier::{RandomMapScriptId, TerrainId, UnitId};
use chariot_io_tools::{ReadArrayExt, ReadExt};
use std::io::SeekFrom;

use std::io::prelude::{Seek, Read};

#[derive(Default, Debug)]
pub struct RandomMapHeader {
    script_id: RandomMapScriptId,
    border_sw: i32,
    border_nw: i32,
    border_ne: i32,
    border_se: i32,
    border_usage: i32,
    water_shape: i32,
    non_base_terrain_id: TerrainId,
    base_zone_coverage: i32,
    base_zone_count: u32,
    terrain_count: u32,
    unit_count: u32,
}

#[derive(Default, Debug)]
pub struct BaseZone {
    base_terrain_id: TerrainId,
    space_between_players: i32,
    start_area_radius: i32,
}

#[derive(Default, Debug)]
pub struct MapTerrain {
    proportion: i32,
    terrain_id: TerrainId,
    clump_count: i32,
    spacing_to_other_terrains: i32,
    placement_zone: i32,
}

#[derive(Default, Debug)]
pub struct MapUnit {
    unit_id: UnitId,
    host_terrain_id: Option<TerrainId>,
    objects_per_group: i32,
    fluctuation: i32,
    groups_per_player: i32,
    group_radius: i32,
    own_at_start: i32,
    set_place_for_all_players: i32,
    min_distance_to_players: i32,
    max_distance_to_players: i32,
}

#[derive(Default, Debug)]
pub struct RandomMap {
    border_sw: i32,
    border_nw: i32,
    border_ne: i32,
    border_se: i32,
    border_usage: i32,
    water_shape: i32,
    non_base_terrain_id: TerrainId,
    base_zone_coverage: i32,
    base_zones: Vec<BaseZone>,
    terrains: Vec<MapTerrain>,
    units: Vec<MapUnit>,
}

pub fn read_random_maps<R: Read + Seek>(stream: &mut R) -> Result<Vec<RandomMap>> {
    let mut random_maps = Vec::new();

    let random_map_count = stream.read_u32()? as usize;
    stream.read_u32()?; // Unused: random map pointer
    for _ in 0..random_map_count {
        // Not certain how useful the header is since most of its information is
        // repeated in the actual random map data; just drop it for now
        read_random_map_header(stream)?;
    }
    for _ in 0..random_map_count {
        random_maps.push(read_random_map(stream)?);
    }

    Ok(random_maps)
}

fn read_map_unit<R: Read>(stream: &mut R) -> Result<MapUnit> {
    let mut unit: MapUnit = Default::default();
    unit.unit_id = required_id!(stream.read_i32()?);
    unit.host_terrain_id = optional_id!(stream.read_i32()?);
    stream.read_i32()?; // Unknown
    unit.objects_per_group = stream.read_i32()?;
    unit.fluctuation = stream.read_i32()?;
    unit.groups_per_player = stream.read_i32()?;
    unit.group_radius = stream.read_i32()?;
    unit.own_at_start = stream.read_i32()?;
    unit.set_place_for_all_players = stream.read_i32()?;
    unit.min_distance_to_players = stream.read_i32()?;
    unit.max_distance_to_players = stream.read_i32()?;
    Ok(unit)
}

fn read_map_terrain<R: Read>(stream: &mut R) -> Result<MapTerrain> {
    let mut terrain: MapTerrain = Default::default();
    terrain.proportion = stream.read_i32()?;
    terrain.terrain_id = required_id!(stream.read_i32()?);
    terrain.clump_count = stream.read_i32()?;
    terrain.spacing_to_other_terrains = stream.read_i32()?;
    terrain.placement_zone = stream.read_i32()?;
    stream.read_i32()?; // Unknown
    Ok(terrain)
}

fn read_base_zone<R: Read + Seek>(stream: &mut R) -> Result<BaseZone> {
    let mut zone: BaseZone = Default::default();
    stream.read_u32()?; // Unknown
    zone.base_terrain_id = required_id!(stream.read_i32()?);
    zone.space_between_players = stream.read_i32()?;
    stream.seek(SeekFrom::Current(20))?; // 20 unknown bytes
    zone.start_area_radius = stream.read_i32()?;
    stream.seek(SeekFrom::Current(8))?; // 8 unknown bytes
    Ok(zone)
}

fn read_random_map<R: Read + Seek>(stream: &mut R) -> Result<RandomMap> {
    let mut map: RandomMap = Default::default();
    map.border_sw = stream.read_i32()?;
    map.border_nw = stream.read_i32()?;
    map.border_ne = stream.read_i32()?;
    map.border_se = stream.read_i32()?;
    map.border_usage = stream.read_i32()?;
    map.water_shape = stream.read_i32()?;
    map.non_base_terrain_id = required_id!(stream.read_i32()?);
    map.base_zone_coverage = stream.read_i32()?;
    stream.read_i32()?; // Unknown

    let base_zone_count = stream.read_u32()? as usize;
    stream.read_u32()?; // Unused: Base zone pointer
    map.base_zones = stream.read_array(base_zone_count, |c| read_base_zone(c))?;

    let terrain_count = stream.read_u32()? as usize;
    stream.read_u32()?; // Unused: Terrain pointer
    map.terrains = stream.read_array(terrain_count, |c| read_map_terrain(c))?;

    let unit_count = stream.read_u32()? as usize;
    stream.read_u32()?; // Unused: Unit pointer
    map.units = stream.read_array(unit_count, |c| read_map_unit(c))?;

    let unknown_count = stream.read_u32()? as i64;
    stream.read_u32()?; // Unused: Unknown pointer
    stream.seek(SeekFrom::Current(24 * unknown_count))?; // Skip unknown data

    Ok(map)
}

fn read_random_map_header<R: Read + Seek>(stream: &mut R) -> Result<RandomMapHeader> {
    let mut header: RandomMapHeader = Default::default();
    header.script_id = required_id!(stream.read_i32()?);
    header.border_sw = stream.read_i32()?;
    header.border_nw = stream.read_i32()?;
    header.border_ne = stream.read_i32()?;
    header.border_se = stream.read_i32()?;
    header.border_usage = stream.read_i32()?;
    header.water_shape = stream.read_i32()?;
    header.non_base_terrain_id = required_id!(stream.read_i32()?);
    header.base_zone_coverage = stream.read_i32()?;
    stream.read_i32()?; // Unknown

    header.base_zone_count = stream.read_u32()?;
    stream.read_i32()?; // Unused: Base zone pointer

    header.terrain_count = stream.read_u32()?;
    stream.read_i32()?; // Unused: Terrain pointer

    header.unit_count = stream.read_u32()?;
    stream.read_i32()?; // Unused: Unit pointer

    stream.read_i32()?; // Unknown count
    stream.read_i32()?; // Unused: unknown pointer
    Ok(header)
}
