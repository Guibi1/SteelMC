use std::sync::Arc;

use async_trait::async_trait;
use nylium::{Nylium, NyliumLogger};
use nylium_adapter::fields::{FieldOptions, FieldValue};
use nylium_adapter::{GameRuleKeys, Global, NyliumServer, Player, PlayerMap};
use steel::SteelServer;
use steel_core::command::sender::CommandSender;
use steel_core::config::STEEL_CONFIG;
use tokio::sync::Mutex;
use tokio_util::task::TaskTracker;

fn main() {
    let logger = NyliumLogger::init();

    Nylium::new(SteelServerNylium::new(), logger).run();
}

#[derive(Clone)]
struct SteelServerNylium {
    server: Arc<Mutex<Option<SteelServer>>>,
    runtime: Arc<tokio::runtime::Runtime>,
    chunk_runtime: Arc<tokio::runtime::Runtime>,
}

impl SteelServerNylium {
    fn new() -> Self {
        Self {
            server: Arc::new(Mutex::new(None)),
            runtime: Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap(),
            ),
            chunk_runtime: Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap(),
            ),
        }
    }
}

impl Global for SteelServerNylium {}

#[async_trait]
impl NyliumServer<SteelConfigKeys, GameRuleKeys> for SteelServerNylium {
    async fn start(&self) {
        let mut steel = SteelServer::new(self.chunk_runtime.clone()).await;
        let this = self.clone();

        self.runtime.spawn(async move {
            let task_tracker = TaskTracker::new();
            steel.start(task_tracker.clone()).await;
            task_tracker.close();
            task_tracker.wait().await;

            *this.server.lock().await = Some(steel);
        });
    }

    async fn stop(&self) {
        if let Some(ref steel) = *self.server.lock().await {
            steel.stop();
        }
    }

    async fn get_players(&self) -> Vec<Player> {
        let mut players = Vec::new();

        if let Some(ref steel) = *self.server.lock().await {
            steel.server.worlds[0]
                .players
                .iter_async(|_k, p| {
                    players.push(Player::new(
                        p.gameprofile.id,
                        p.gameprofile.name.clone(),
                        PlayerMap::Overworld,
                        true,
                    ));
                    false
                })
                .await;
        }

        players
    }

    async fn run_command(&self, command: &str) {
        if let Some(ref steel) = *self.server.lock().await {
            steel.server.command_dispatcher.read().handle_command(
                CommandSender::Console,
                command.to_string(),
                &steel.server,
            );
        }
    }

    fn get_config(&self) -> Box<[FieldOptions<SteelConfigKeys>]> {
        Box::new([
            FieldOptions::new_number(
                SteelConfigKeys::ServerPort,
                "Server Port",
                Some(1),
                Some(65535),
            ),
            FieldOptions::new_string(SteelConfigKeys::Seed, "World Seed"),
            FieldOptions::new_number(
                SteelConfigKeys::MaxPlayers,
                "Max Players",
                Some(1),
                Some(1000),
            ),
            FieldOptions::new_number(
                SteelConfigKeys::ViewDistance,
                "View Distance",
                Some(1),
                Some(64),
            ),
            FieldOptions::new_number(
                SteelConfigKeys::SimulationDistance,
                "Simulation Distance",
                Some(1),
                Some(32),
            ),
            FieldOptions::new_bool(SteelConfigKeys::OnlineMode, "Online Mode", "online_mode"),
            FieldOptions::new_bool(SteelConfigKeys::Encryption, "Encryption", "encryption"),
            FieldOptions::new_string(SteelConfigKeys::Motd, "Message of the Day"),
            FieldOptions::new_bool(SteelConfigKeys::UseFavicon, "Use Favicon", "use_favicon"),
            FieldOptions::new_string(SteelConfigKeys::Favicon, "Favicon Path"),
            FieldOptions::new_bool(
                SteelConfigKeys::EnforceSecureChat,
                "Enforce Secure Chat",
                "enforce_secure_chat",
            ),
        ])
    }

    fn get_config_value(&self, key: SteelConfigKeys) -> FieldValue {
        match key {
            SteelConfigKeys::ServerPort => FieldValue::Number(STEEL_CONFIG.server_port.into()),
            SteelConfigKeys::Seed => FieldValue::String(STEEL_CONFIG.seed.clone()),
            SteelConfigKeys::MaxPlayers => FieldValue::Number(STEEL_CONFIG.max_players.into()),
            SteelConfigKeys::ViewDistance => FieldValue::Number(STEEL_CONFIG.view_distance.into()),
            SteelConfigKeys::SimulationDistance => {
                FieldValue::Number(STEEL_CONFIG.simulation_distance.into())
            }
            SteelConfigKeys::OnlineMode => FieldValue::Boolean(STEEL_CONFIG.online_mode),
            SteelConfigKeys::Encryption => FieldValue::Boolean(STEEL_CONFIG.encryption),
            SteelConfigKeys::Motd => FieldValue::String(STEEL_CONFIG.motd.clone()),
            SteelConfigKeys::UseFavicon => FieldValue::Boolean(STEEL_CONFIG.use_favicon),
            SteelConfigKeys::Favicon => FieldValue::String(STEEL_CONFIG.favicon.clone()),
            SteelConfigKeys::EnforceSecureChat => {
                FieldValue::Boolean(STEEL_CONFIG.enforce_secure_chat)
            }
        }
    }

    fn set_config_value(&self, _key: SteelConfigKeys, _value: FieldValue) {
        // TODO: Allow settings to be changed at runtime
    }

    fn get_gamerules(&self) -> Box<[FieldOptions<GameRuleKeys>]> {
        GameRuleKeys::get_gamerules()
    }

    fn get_gamerule_value(&self, key: GameRuleKeys) -> FieldValue {
        match key {
            GameRuleKeys::AnnounceAdvancements => FieldValue::Boolean(true),
            GameRuleKeys::BlockExplosionDropDecay => FieldValue::Boolean(true),
            GameRuleKeys::CommandBlockOutput => FieldValue::Boolean(true),
            GameRuleKeys::CommandModificationBlockLimit => FieldValue::Number(32768),
            GameRuleKeys::DisableElytraMovementCheck => FieldValue::Boolean(false),
            GameRuleKeys::DisableRaids => FieldValue::Boolean(false),
            GameRuleKeys::DoDaylightCycle => FieldValue::Boolean(true),
            GameRuleKeys::DoEntityDrops => FieldValue::Boolean(true),
            GameRuleKeys::DoFireTick => FieldValue::Boolean(true),
            GameRuleKeys::DoImmediateRespawn => FieldValue::Boolean(false),
            GameRuleKeys::DoInsomnia => FieldValue::Boolean(true),
            GameRuleKeys::DoLimitedCrafting => FieldValue::Boolean(false),
            GameRuleKeys::DoMobLoot => FieldValue::Boolean(true),
            GameRuleKeys::DoMobSpawning => FieldValue::Boolean(true),
            GameRuleKeys::DoPatrolSpawning => FieldValue::Boolean(true),
            GameRuleKeys::DoTileDrops => FieldValue::Boolean(true),
            GameRuleKeys::DoTraderSpawning => FieldValue::Boolean(true),
            GameRuleKeys::DoVinesSpread => FieldValue::Boolean(true),
            GameRuleKeys::DoWardenSpawning => FieldValue::Boolean(true),
            GameRuleKeys::DoWeatherCycle => FieldValue::Boolean(true),
            GameRuleKeys::DrowningDamage => FieldValue::Boolean(true),
            GameRuleKeys::EnderPearlsVanishOnDeath => FieldValue::Boolean(true),
            GameRuleKeys::FallDamage => FieldValue::Boolean(true),
            GameRuleKeys::FireDamage => FieldValue::Boolean(true),
            GameRuleKeys::ForgiveDeadPlayers => FieldValue::Boolean(true),
            GameRuleKeys::FreezeDamage => FieldValue::Boolean(true),
            GameRuleKeys::GlobalSoundEvents => FieldValue::Boolean(true),
            GameRuleKeys::KeepInventory => FieldValue::Boolean(false),
            GameRuleKeys::LavaSourceConversion => FieldValue::Boolean(false),
            GameRuleKeys::LogAdminCommands => FieldValue::Boolean(true),
            GameRuleKeys::MaxCommandChainLength => FieldValue::Number(65536),
            GameRuleKeys::MaxCommandForkCount => FieldValue::Number(65536),
            GameRuleKeys::MaxEntityCramming => FieldValue::Number(24),
            GameRuleKeys::MobExplosionDropDecay => FieldValue::Boolean(true),
            GameRuleKeys::MobGriefing => FieldValue::Boolean(true),
            GameRuleKeys::NaturalRegeneration => FieldValue::Boolean(true),
            GameRuleKeys::PlayersNetherPortalCreativeDelay => FieldValue::Number(1),
            GameRuleKeys::PlayersNetherPortalDefaultDelay => FieldValue::Number(80),
            GameRuleKeys::PlayersSleepingPercentage => FieldValue::Number(100),
            GameRuleKeys::ProjectilesCanBreakBlocks => FieldValue::Boolean(true),
            GameRuleKeys::RandomTickSpeed => FieldValue::Number(3),
            GameRuleKeys::ReducedDebugInfo => FieldValue::Boolean(false),
            GameRuleKeys::SendCommandFeedback => FieldValue::Boolean(true),
            GameRuleKeys::ShowDeathMessages => FieldValue::Boolean(true),
            GameRuleKeys::SnowAccumulationHeight => FieldValue::Number(1),
            GameRuleKeys::SpawnChunkRadius => FieldValue::Number(2),
            GameRuleKeys::SpawnRadius => FieldValue::Number(10),
            GameRuleKeys::SpectatorsGenerateChunks => FieldValue::Boolean(true),
            GameRuleKeys::TntExplosionDropDecay => FieldValue::Boolean(false),
            GameRuleKeys::UniversalAnger => FieldValue::Boolean(false),
            GameRuleKeys::WaterSourceConversion => FieldValue::Boolean(true),
        }
    }

    fn set_gamerule_value(&self, _key: GameRuleKeys, _value: FieldValue) {
        // TODO: Allow gamerules to be changed at runtime
    }
}

#[derive(Clone, Copy)]
enum SteelConfigKeys {
    ServerPort,
    Seed,
    MaxPlayers,
    ViewDistance,
    SimulationDistance,
    OnlineMode,
    Encryption,
    Motd,
    UseFavicon,
    Favicon,
    EnforceSecureChat,
}
