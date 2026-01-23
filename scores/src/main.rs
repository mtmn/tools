use chrono::{DateTime, Utc};
use prettytable::{Cell, Row, Table, format};
use serde::Deserialize;
use std::env;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct Match {
    #[serde(default)]
    #[serde(rename = "utcDate")]
    utc_date: Option<String>,
    #[serde(default)]
    status: String,
    #[serde(default)]
    matchday: Option<u32>,
    #[serde(rename = "homeTeam")]
    home_team: Team,
    #[serde(rename = "awayTeam")]
    away_team: Team,
    score: Score,
}

#[derive(Debug, Deserialize)]
struct Team {
    #[serde(default)]
    name: String,
    #[serde(default)]
    tla: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Score {
    #[serde(default)]
    #[serde(rename = "fullTime")]
    full_time: ScoreDetail,
}

#[derive(Debug, Deserialize, Default)]
struct ScoreDetail {
    #[serde(default)]
    home: Option<u32>,
    #[serde(default)]
    away: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct MatchesResponse {
    matches: Vec<Match>,
}

#[derive(Debug, Deserialize)]
struct StandingsResponse {
    standings: Vec<Standing>,
}

#[derive(Debug, Deserialize)]
struct Standing {
    #[serde(default)]
    #[serde(rename = "type")]
    standing_type: String,
    table: Vec<TableEntry>,
}

#[derive(Debug, Deserialize)]
struct TableEntry {
    position: u32,
    team: TeamInfo,
    #[serde(rename = "playedGames")]
    played_games: u32,
    won: u32,
    draw: u32,
    lost: u32,
    points: u32,
    #[serde(rename = "goalsFor")]
    goals_for: u32,
    #[serde(rename = "goalsAgainst")]
    goals_against: u32,
    #[serde(rename = "goalDifference")]
    goal_difference: i32,
}

#[derive(Debug, Deserialize)]
struct TeamInfo {
    #[serde(default)]
    name: String,
    #[serde(default)]
    tla: Option<String>,
}

#[derive(Debug, Clone)]
enum League {
    Premier,
    Championship,
    LaLiga,
    Champions,
}

impl League {
    fn code(&self) -> &str {
        match self {
            League::Premier => "PL",
            League::Championship => "ELC",
            League::LaLiga => "PD",
            League::Champions => "CL",
        }
    }

    fn from_arg(arg: &str) -> Option<Self> {
        match arg.to_lowercase().as_str() {
            "epl" | "pl" | "premier" => Some(League::Premier),
            "championship" | "elc" | "champ" => Some(League::Championship),
            "laliga" | "liga" | "pd" => Some(League::LaLiga),
            "cl" | "ucl" | "champions" => Some(League::Champions),
            _ => None,
        }
    }
}

async fn fetch_matches(
    client: &reqwest::Client,
    api_token: &str,
    league: &League,
) -> Result<MatchesResponse, Box<dyn Error>> {
    let url = format!(
        "https://api.football-data.org/v4/competitions/{}/matches",
        league.code()
    );

    let response = client
        .get(&url)
        .header("X-Auth-Token", api_token)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()).into());
    }

    Ok(response.json::<MatchesResponse>().await?)
}

async fn fetch_standings(
    client: &reqwest::Client,
    api_token: &str,
    league: &League,
) -> Result<StandingsResponse, Box<dyn Error>> {
    let url = format!(
        "https://api.football-data.org/v4/competitions/{}/standings",
        league.code()
    );

    let response = client
        .get(&url)
        .header("X-Auth-Token", api_token)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()).into());
    }

    Ok(response.json::<StandingsResponse>().await?)
}

fn filter_matches(matches: Vec<Match>, team_filter: Option<&str>, show_all: bool) -> Vec<Match> {
    let now = Utc::now();
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();

    matches
        .into_iter()
        .filter(|m| {
            if !show_all {
                if let Some(date_str) = &m.utc_date {
                    let match_time = DateTime::parse_from_rfc3339(date_str)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc));

                    if let Some(mt) = match_time {
                        if mt < today_start {
                            return false;
                        }
                    }
                } else {
                    return false;
                }
            }

            if let Some(team) = team_filter {
                let team_lower = team.to_lowercase();
                let home_match = m.home_team.name.to_lowercase().contains(&team_lower)
                    || m.home_team
                        .tla
                        .as_ref()
                        .is_some_and(|t| t.to_lowercase().contains(&team_lower));
                let away_match = m.away_team.name.to_lowercase().contains(&team_lower)
                    || m.away_team
                        .tla
                        .as_ref()
                        .is_some_and(|t| t.to_lowercase().contains(&team_lower));

                home_match || away_match
            } else {
                true
            }
        })
        .collect()
}

fn display_table(mut matches: Vec<Match>) {
    if matches.is_empty() {
        return;
    }

    matches.sort_by(|a, b| {
        let a_time = a
            .utc_date
            .as_ref()
            .and_then(|d| DateTime::parse_from_rfc3339(d).ok());
        let b_time = b
            .utc_date
            .as_ref()
            .and_then(|d| DateTime::parse_from_rfc3339(d).ok());
        b_time.cmp(&a_time)
    });

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_BOX_CHARS);

    table.add_row(Row::new(vec![
        Cell::new("MD").style_spec("Fb"),
        Cell::new("Date & Time").style_spec("Fb"),
        Cell::new("Home Team").style_spec("Fb"),
        Cell::new("     Score     ").style_spec("Fb"),
        Cell::new("Away Team").style_spec("Fb"),
        Cell::new("Status").style_spec("Fb"),
    ]));

    for m in &matches {
        let matchday = m.matchday.map_or("-".to_string(), |md| md.to_string());

        let date = if let Some(date_str) = &m.utc_date {
            DateTime::parse_from_rfc3339(date_str).ok().map_or_else(
                || date_str.clone(),
                |dt| {
                    dt.with_timezone(&Utc)
                        .format("%b %d, %Y %H:%M UTC")
                        .to_string()
                },
            )
        } else {
            "TBD".to_string()
        };

        let score = match (m.score.full_time.home, m.score.full_time.away) {
            (Some(h), Some(a)) => format!("     {h} - {a}     "),
            _ => "      vs      ".to_string(),
        };

        let status_icon = match m.status.as_str() {
            "FINISHED" => "✓",
            "SCHEDULED" => "⏰",
            "IN_PLAY" => "▶",
            _ => "•",
        };

        table.add_row(Row::new(vec![
            Cell::new(&matchday),
            Cell::new(&date),
            Cell::new(&m.home_team.name).style_spec("r"),
            Cell::new(&score).style_spec("Fyc"),
            Cell::new(&m.away_team.name),
            Cell::new(&format!("{status_icon} {}", m.status)),
        ]));
    }

    table.printstd();
}

fn display_standings(standings_response: StandingsResponse) {
    for standing in standings_response.standings {
        if standing.table.is_empty() {
            continue;
        }

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BOX_CHARS);

        table.add_row(Row::new(vec![
            Cell::new("Pos").style_spec("Fb"),
            Cell::new("Team").style_spec("Fb"),
            Cell::new("P").style_spec("Fb"),
            Cell::new("W").style_spec("Fb"),
            Cell::new("D").style_spec("Fb"),
            Cell::new("L").style_spec("Fb"),
            Cell::new("GF").style_spec("Fb"),
            Cell::new("GA").style_spec("Fb"),
            Cell::new("GD").style_spec("Fb"),
            Cell::new("Pts").style_spec("Fb"),
        ]));

        for entry in &standing.table {
            let gd_str = if entry.goal_difference >= 0 {
                format!("+{}", entry.goal_difference)
            } else {
                entry.goal_difference.to_string()
            };

            table.add_row(Row::new(vec![
                Cell::new(&entry.position.to_string()).style_spec("c"),
                Cell::new(&entry.team.name),
                Cell::new(&entry.played_games.to_string()).style_spec("c"),
                Cell::new(&entry.won.to_string()).style_spec("c"),
                Cell::new(&entry.draw.to_string()).style_spec("c"),
                Cell::new(&entry.lost.to_string()).style_spec("c"),
                Cell::new(&entry.goals_for.to_string()).style_spec("c"),
                Cell::new(&entry.goals_against.to_string()).style_spec("c"),
                Cell::new(&gd_str).style_spec("c"),
                Cell::new(&entry.points.to_string()).style_spec("Fbc"),
            ]));
        }

        table.printstd();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    // Check for help first
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("USAGE: scores [OPTIONS]");
        println!();
        println!("OPTIONS:");
        println!("  --help, -h          Show this help message");
        println!("  --standings, --table Show league standings/table");
        println!();
        println!("LEAGUES:");
        println!("  --epl, --pl         Premier League (default)");
        println!("  --championship      Championship");
        println!("  --laliga            La Liga");
        println!("  --cl, --ucl         Champions League");
        println!();
        println!("FILTERS:");
        println!("  --all               Show all matches (past and future)");
        println!("  --<team>            Filter by team name or abbreviation");
        println!();
        println!("EXAMPLES:");
        println!("  scores                        Show upcoming Premier League matches");
        println!("  scores --standings            Show Premier League table");
        println!("  scores --epl --arsenal        Show upcoming Arsenal matches");
        println!("  scores --epl --ars --all      Show all Arsenal matches this season");
        println!("  scores --laliga --table       Show La Liga standings");
        println!("  scores --cl                   Show upcoming Champions League matches");
        println!("  scores --cl --bayern --all    Show all Bayern CL matches");
        println!();
        println!("ENVIRONMENT:");
        println!("  FOOTBALL_DATA_API_TOKEN       API token from football-data.org");
        return Ok(());
    }

    let mut league = League::Premier;
    let mut team_filter: Option<String> = None;
    let mut show_all = false;
    let mut show_standings = false;

    for arg in &args {
        if arg == "--help" || arg == "-h" {
            continue;
        }

        let value = arg.trim_start_matches("--");
        if value == "all" {
            show_all = true;
        } else if value == "standings" || value == "table" {
            show_standings = true;
        } else if let Some(l) = League::from_arg(value) {
            league = l;
        } else if value.parse::<u32>().is_err() {
            team_filter = Some(value.to_string());
        }
    }

    let api_token = env::var("FOOTBALL_DATA_API_TOKEN").unwrap_or_default();
    let client = reqwest::Client::new();

    if show_standings {
        let standings = fetch_standings(&client, &api_token, &league).await?;
        display_standings(standings);
    } else {
        let response = fetch_matches(&client, &api_token, &league).await?;
        let filtered = filter_matches(response.matches, team_filter.as_deref(), show_all);
        display_table(filtered);
    }

    Ok(())
}
