use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crate::prelude::*;
use crate::PlayingState;

pub struct NodeEditorPlugin;

impl Plugin for NodeEditorPlugin {
    fn build(&self, app: &mut App) {
        // We will let the inspector add the plugin
        #[cfg(not(feature = "dev"))]
        {
            app.add_plugins(EguiPlugin);
        }
        app.init_resource::<SnarlContainer>()
            .add_event::<NodeOutputTrigger>()
            .add_event::<NodeTrigger>()
            .add_event::<WorldEvent>()
            .add_systems(Update, node_editor.run_if(in_state(PlayingState::Editor)))
            .add_systems(
                Update,
                (do_world_events, activate_nodes).run_if(in_state(PlayingState::ShootyTime)),
            );
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Node {
    OnShoot,
    SpawnBullet,
    DealDmg,
    Spread,
    Repeating,
    Explosion,
}

#[derive(Clone, Debug, Default)]
pub struct NodeEventData {
    pub loc: Option<Vec2>,
    pub dir: Option<Vec2>,
    pub target: Option<Entity>,
}

#[derive(Event, Debug)]
pub struct NodeOutputTrigger {
    pub data: NodeEventData,
    pub node: egui_snarl::NodeId,
    pub output_index: usize,
}

#[derive(Event, Debug)]
struct NodeTrigger {
    data: NodeEventData,
    node: egui_snarl::NodeId,
}

#[derive(Event, Debug)]
pub enum WorldEvent {
    SpawnBullet {
        loc: Option<Vec2>,
        dir: Option<Vec2>,
        id: egui_snarl::NodeId,
    },
    DealDmg {
        target: Option<Entity>,
        id: egui_snarl::NodeId,
    },
    Spread {
        data: NodeEventData,
    },
}

fn do_world_events(
    mut node_trigger: EventReader<NodeTrigger>,
    mut world: EventWriter<WorldEvent>,
    snarl: Res<SnarlContainer>,
) {
    for event in node_trigger.read() {
        let Some(node) = snarl.snarl.get_node(event.node) else {
            continue;
        };
        match node {
            Node::SpawnBullet => {
                world.send(WorldEvent::SpawnBullet {
                    loc: event.data.loc,
                    dir: event.data.dir,
                    id: event.node,
                });
            }
            Node::DealDmg => {
                world.send(WorldEvent::DealDmg {
                    target: event.data.target,
                    id: event.node,
                });
            }
            Node::Spread => {
                world.send(WorldEvent::Spread {
                    data: event.data.clone(),
                });
            }
            Node::OnShoot => {}
            _ => unimplemented!(),
        }
    }
}
fn activate_nodes(
    mut output_triggers: EventReader<NodeOutputTrigger>,
    mut node_triggers: EventWriter<NodeTrigger>,
    snarl: Res<SnarlContainer>,
) {
    for event in output_triggers.read() {
        let pin_id = egui_snarl::OutPinId {
            node: event.node,
            output: event.output_index,
        };
        let pin = snarl.snarl.out_pin(pin_id);

        for connected in pin.remotes {
            node_triggers.send(NodeTrigger {
                data: event.data.clone(),
                node: connected.node,
            });
        }
    }
}

#[derive(Resource)]
pub struct SnarlContainer {
    pub snarl: egui_snarl::Snarl<Node>,
    pub shoot_trigger: egui_snarl::NodeId,
}

impl Default for SnarlContainer {
    fn default() -> Self {
        let mut snarl = egui_snarl::Snarl::new();

        let shoot = snarl.insert_node(egui::Pos2::new(0.0, 0.0), Node::OnShoot);
        let bullet = snarl.insert_node(egui::Pos2::new(150.0, 0.0), Node::SpawnBullet);
        snarl.insert_node(egui::Pos2::new(150.0, 0.0), Node::SpawnBullet);
        snarl.insert_node(egui::Pos2::new(150.0, 0.0), Node::SpawnBullet);
        snarl.insert_node(egui::Pos2::new(-150.0, 0.0), Node::Spread);
        let dmg = snarl.insert_node(egui::Pos2::new(150.0, 100.0), Node::DealDmg);

        snarl.connect(
            egui_snarl::OutPinId {
                node: shoot,
                output: 0,
            },
            egui_snarl::InPinId {
                node: bullet,
                input: 0,
            },
        );
        snarl.connect(
            egui_snarl::OutPinId {
                node: bullet,
                output: 0,
            },
            egui_snarl::InPinId {
                node: dmg,
                input: 0,
            },
        );

        Self {
            snarl,
            shoot_trigger: shoot,
        }
    }
}

struct Viewer;

impl egui_snarl::ui::SnarlViewer<Node> for Viewer {
    fn title(&mut self, node: &Node) -> String {
        match node {
            Node::OnShoot => String::from("On shoot"),
            Node::SpawnBullet => String::from("Spawn Bullet"),
            Node::Explosion => String::from("Spawn Explosion"),
            Node::Repeating => String::from("Repeat"),
            Node::DealDmg => String::from("Dmg"),
            Node::Spread => String::from("Spread"),
        }
    }
    fn outputs(&mut self, node: &Node) -> usize {
        match node {
            Node::SpawnBullet => 2,
            Node::Explosion | Node::Repeating | Node::DealDmg | Node::OnShoot | Node::Spread => 1,
        }
    }
    fn inputs(&mut self, node: &Node) -> usize {
        match node {
            Node::OnShoot => 0,
            Node::SpawnBullet
            | Node::Explosion
            | Node::Repeating
            | Node::DealDmg
            | Node::Spread => 1,
        }
    }
    fn show_input(
        &mut self,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut egui_snarl::Snarl<Node>,
    ) -> egui_snarl::ui::PinInfo {
        if let Some(node) = snarl.get_node(pin.id.node) {
            let label = match node {
                Node::OnShoot => "",
                Node::SpawnBullet => "Spawn",
                Node::Explosion => "Spawn",
                Node::Repeating => "Event",
                Node::DealDmg => "Target",
                Node::Spread => "",
            };
            ui.label(label);
        }

        egui_snarl::ui::PinInfo::triangle().with_fill(if pin.remotes.is_empty() {
            egui::Color32::DARK_GREEN
        } else {
            egui::Color32::GREEN
        })
    }
    fn show_output(
        &mut self,
        pin: &egui_snarl::OutPin,
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut egui_snarl::Snarl<Node>,
    ) -> egui_snarl::ui::PinInfo {
        if let Some(node) = snarl.get_node(pin.id.node) {
            let label = match node {
                Node::OnShoot => "Hit",
                Node::SpawnBullet => ["Hit", "Despawned"][pin.id.output],
                Node::Explosion => "Hit",
                Node::Repeating => "Event",
                Node::DealDmg => "Fatal",
                Node::Spread => "",
            };
            ui.label(label);
        }

        egui_snarl::ui::PinInfo::circle().with_fill(if pin.remotes.is_empty() {
            egui::Color32::DARK_GREEN
        } else {
            egui::Color32::GREEN
        })
    }
    fn output_color(
        &mut self,
        pin: &egui_snarl::OutPin,
        style: &egui::Style,
        snarl: &mut egui_snarl::Snarl<Node>,
    ) -> egui::Color32 {
        egui::Color32::GREEN
    }
    fn input_color(
        &mut self,
        pin: &egui_snarl::InPin,
        style: &egui::Style,
        snarl: &mut egui_snarl::Snarl<Node>,
    ) -> egui::Color32 {
        egui::Color32::GREEN
    }
    fn has_body(&mut self, node: &Node) -> bool {
        true
    }
    fn show_body(
        &mut self,
        node_id: egui_snarl::NodeId,
        inputs: &[egui_snarl::InPin],
        outputs: &[egui_snarl::OutPin],
        ui: &mut egui::Ui,
        scale: f32,
        snarl: &mut egui_snarl::Snarl<Node>,
    ) {
        let Some(node) = snarl.get_node(node_id) else {
            return;
        };
        match node {
            _ => {}
        }
    }
    fn connect(
        &mut self,
        from: &egui_snarl::OutPin,
        to: &egui_snarl::InPin,
        snarl: &mut egui_snarl::Snarl<Node>,
    ) {
        let from_node_id = from.id.node;
        // We should be disalloving all loops so this shouldnt hang
        let mut stack = vec![to.id.node];
        while let Some(node_id) = stack.pop() {
            if node_id == from_node_id {
                // TODO: Play some sort of error sound
                return;
            }
            let Some(node) = snarl.get_node(node_id) else {
                bevy::log::warn!("Node not found");
                continue;
            };
            let count = self.outputs(node);
            for i in 0..count {
                let pin = snarl.out_pin(egui_snarl::OutPinId {
                    node: node_id,
                    output: i,
                });
                for pin in pin.remotes {
                    stack.push(pin.node);
                }
            }
        }

        // snarl.drop_outputs(from.id);
        snarl.drop_inputs(to.id);
        snarl.connect(from.id, to.id);
    }
}

fn node_editor(mut ctx: EguiContexts, mut snarl: ResMut<SnarlContainer>) {
    egui::Window::new("Node Editor")
        .default_size((1500.0, 900.0))
        .show(ctx.ctx_mut(), |ui| {
            let style = egui_snarl::ui::SnarlStyle::new();

            snarl.snarl.show(&mut Viewer, &style, "node_editor", ui);
        });
}
