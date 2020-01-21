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

use crate::error::Result;
use identifier::CivilizationId;

use chariot_io_tools::{ReadArrayExt, ReadExt};

use std::io::{Read, Seek};

#[derive(Default, Debug)]
pub struct PlayerData {
    version: f32,
    pub player_names: Vec<String>,
    pub player_civs: Vec<PlayerCivilization>,
    pub conquest_victory: bool,
    unknown1: Vec<u8>,
    pub original_file_name: String,

    pub instructions: String,
    pub hints: String,
    pub victory: String,
    pub loss: String,
    pub history: String,

    pub pre_game_cinematic_file_name: String,
    pub victory_cinematic_file_name: String,
    pub loss_cinematic_file_name: String,
    pub background_file_name: String,

    pub preview_thumbnail: PreviewThumbnail,

    pub ai_names: Vec<String>,
    pub city_names: Vec<String>,
    pub personality_names: Vec<String>,
    pub ai_script_configs: Vec<AiScriptConfig>,
    pub ai_types: Vec<u8>,

    pub player_starting_resources: Vec<PlayerStartingResources>,

    pub victory_conditions: VictoryConditions,
    pub diplomacy: Diplomacy,

    pub allied_victory: Vec<u32>,
    pub disabled_research_ids: Vec<Vec<u32>>,
    unused1: u32,
    unused2: u32,

    pub all_techs: bool,
    pub starting_ages: Vec<u32>,
}

#[derive(Default, Debug)]
pub struct PlayerCivilization {
    state: u32, // enabled flag?
    type_id: u32, // human/ai?
    pub civilization_id: CivilizationId,
    unknown1: u32,
}

#[derive(Default, Debug)]
pub struct PlayerStartingResources {
    gold: u32,
    wood: u32,
    food: u32,
    stone: u32,
}

#[derive(Default, Debug)]
pub struct PreviewThumbnail {
    included: bool,
    width: u32,
    height: u32,
    unknown1: Vec<u8>,
    pixel_data_length: u32,
    unknown2: Vec<u8>,
    pixel_data: Vec<u8>,
}

#[derive(Default, Debug)]
pub struct AiScriptConfig {
    ai_file_name: String,
    city_file_name: String,
    personality_file_name: String,
}

#[derive(Default, Debug)]
pub struct VictoryConditions {
    conquest_required: bool,
    unused1: u32,
    required_relic_count: u32,
    unused2: u32,
    required_exploration_percent: u32,
    unused3: u32,
    all_conditions_required: bool,
    victory_mode: u32,
    score_required: u32,
    timed_game_time: u32,
}

#[derive(Default, Debug)]
pub struct Diplomacy {
    stances: Vec<Vec<u32>>,
    individual_victory: Vec<Vec<u32>>,
}

const PLAYER_DATA_UNKNOWN_1_LENGTH: usize = 8;
const THUMBNAIL_UNKNOWN_1_LENGTH: usize = 22;
const THUMBNAIL_UNKNOWN_2_LENGTH: usize = 16;

impl PlayerData {
    // TODO: Implement writing

    pub fn read_from_stream<S: Read + Seek>(stream: &mut S) -> Result<PlayerData> {
        let mut data: PlayerData = Default::default();
        data.version = stream.read_f32()?;
        data.player_names = stream.read_array(16, |s| s.read_sized_str(256))?;
        data.player_civs = stream.read_array(16, |s| read_civilization(s))?;
        data.conquest_victory = (stream.read_u8()?) != 0;

        data.unknown1 = vec![0u8; PLAYER_DATA_UNKNOWN_1_LENGTH];
        stream.read_exact(&mut data.unknown1)?;

        data.original_file_name = read_pascal_string(stream)?;
        data.instructions = read_pascal_string(stream)?;
        data.hints = read_pascal_string(stream)?;
        data.victory = read_pascal_string(stream)?;
        data.loss = read_pascal_string(stream)?;
        data.history = read_pascal_string(stream)?;

        data.pre_game_cinematic_file_name = read_pascal_string(stream)?;
        data.victory_cinematic_file_name = read_pascal_string(stream)?;
        data.loss_cinematic_file_name = read_pascal_string(stream)?;
        data.background_file_name = read_pascal_string(stream)?;

        data.preview_thumbnail = read_preview_thumbnail(stream)?;

        data.ai_names = stream.read_array(16, |s| read_pascal_string(s))?;
        data.city_names = stream.read_array(16, |s| read_pascal_string(s))?;
        data.personality_names = stream.read_array(16, |s| read_pascal_string(s))?;
        data.ai_script_configs = stream.read_array(16, |s| read_ai_script_config(s))?;
        data.ai_types = stream.read_array(4, |s| s.read_u8())?;

        data.player_starting_resources = stream.read_array(16, |s| read_player_starting_resources(s))?;
        stream.read_i32()?; // separator (-1)

        data.victory_conditions = read_victory_conditions(stream)?;
        data.diplomacy = read_diplomacy(stream)?;
        stream.read_i32()?; // separator (-1)

        data.allied_victory = stream.read_array(16, |s| s.read_u32())?;
        data.disabled_research_ids = stream.read_array(16, |s| s.read_array(20, |s2| s2.read_u32()))?;

        data.unused1 = stream.read_u32()?;
        data.unused2 = stream.read_u32()?;

        data.all_techs = stream.read_u32()? != 0;
        data.starting_ages = stream.read_array(16, |s| s.read_u32())?;
        stream.read_i32()?; // separator (-1)

        Ok(data)
    }
}

fn read_civilization<S: Read + Seek>(stream: &mut S) -> Result<PlayerCivilization> {
    Ok(PlayerCivilization {
        state: stream.read_u32()?,
        type_id: stream.read_u32()?,
        civilization_id: required_id!(stream.read_i32()?),
        unknown1: stream.read_u32()?,
    })
}

fn read_preview_thumbnail<S: Read + Seek>(stream: &mut S) -> Result<PreviewThumbnail> {
    let mut thumb: PreviewThumbnail = Default::default();
    thumb.included = stream.read_u32()? != 0;
    thumb.width = stream.read_u32()?;
    thumb.height = stream.read_u32()?;

    if thumb.included {
        thumb.unknown1 = vec![0u8; THUMBNAIL_UNKNOWN_1_LENGTH];
        stream.read_exact(&mut thumb.unknown1)?;

        thumb.pixel_data_length = stream.read_u32()? - 40;

        thumb.unknown2 = vec![0u8; THUMBNAIL_UNKNOWN_2_LENGTH];
        stream.read_exact(&mut thumb.unknown2)?;

        thumb.pixel_data = vec![0u8; thumb.pixel_data_length as usize];
        stream.read_exact(&mut thumb.pixel_data)?;
    } else {
        thumb.unknown1 = vec![0u8; 2];
        stream.read_exact(&mut thumb.unknown1)?;
    }

    Ok(thumb)
}

fn read_ai_script_config<S: Read>(stream: &mut S) -> Result<AiScriptConfig> {
    let ai_len = stream.read_u32()? as usize;
    let city_len = stream.read_u32()? as usize;
    let per_len = stream.read_u32()? as usize;

    Ok(AiScriptConfig {
        ai_file_name: stream.read_sized_str(ai_len)?,
        city_file_name: stream.read_sized_str(city_len)?,
        personality_file_name: stream.read_sized_str(per_len)?,
    })
}

fn read_player_starting_resources<S: Read>(stream: &mut S) -> Result<PlayerStartingResources> {
    Ok(PlayerStartingResources {
        gold: stream.read_u32()?,
        wood: stream.read_u32()?,
        food: stream.read_u32()?,
        stone: stream.read_u32()?,
    })
}

fn read_victory_conditions<S: Read>(stream: &mut S) -> Result<VictoryConditions> {
    Ok(VictoryConditions {
        conquest_required: stream.read_u32()? != 0,
        unused1: stream.read_u32()?,
        required_relic_count: stream.read_u32()?,
        unused2: stream.read_u32()?,
        required_exploration_percent: stream.read_u32()?,
        unused3: stream.read_u32()?,
        all_conditions_required: stream.read_u32()? != 0,
        victory_mode: stream.read_u32()?,
        score_required: stream.read_u32()?,
        timed_game_time: stream.read_u32()?,
    })
}

fn read_diplomacy<S: Read>(stream: &mut S) -> Result<Diplomacy> {
    Ok(Diplomacy {
        stances: stream.read_array(16, |s| s.read_array(16, |s2| s2.read_u32()))?,
        individual_victory: stream.read_array(16, |s| s.read_array(180, |s2| s2.read_u32()))?,
    })
}

fn read_pascal_string<S: Read>(stream: &mut S) -> Result<String> {
    let length = stream.read_u16()? as usize;
    Ok(stream.read_sized_str(length)?)
}
