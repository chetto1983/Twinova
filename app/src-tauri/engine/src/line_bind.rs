//! Where the line meets the PLC (spec 2026-11-12-st-vscode-design.md, "The PLC's online change"): the scene fixes at
//! Build which address each sensor, scanner, sink, conveyor and field feedback uses (`Wiring`); the program fixes which
//! of those it declares (`Ports`). Build binds them once, and an online change binds them again for the new program.

use twinova_physics::layout::{Layout, SensorKind};
use twinova_project::Project;

use crate::controller::{Controller, Port};
use crate::field::Field;

/// A conveyor's signals at the PLC: `run` is read whether the program declares it or not (truST reads an undeclared
/// output as zero), the rest are written only when declared.
pub(crate) struct BeltPorts {
    pub run: Option<Port>,
    pub running: Option<Port>,
    pub velocity: Option<Port>,
    pub encoder: Option<Port>,
    pub pulse: Option<Port>,
}

/// The PLC inputs the line writes and the outputs it reads, for one program.
pub(crate) struct Ports {
    pub belts: Vec<BeltPorts>,
    /// Per layout sensor: `detected`, `color` or `distance`.
    pub sensors: Vec<Option<Port>>,
    /// Per layout scanner: `read`, `count` and `code`.
    pub scanners: Vec<[Option<Port>; 3]>,
    /// Per layout sink: `count`.
    pub sinks: Vec<Option<Port>>,
    /// The field's feedback the program declares: the address as the field names it, and the controller's.
    pub feedback: Vec<(String, Port)>,
}

impl Ports {
    /// Every controller input the line writes.
    pub fn inputs(&self) -> Vec<&Port> {
        let belts =
            (self.belts.iter()).flat_map(|b| [&b.running, &b.velocity, &b.encoder, &b.pulse]);
        let scanners = self.scanners.iter().flatten();
        let feedback = self.feedback.iter().map(|(_, port)| port);
        (belts
            .chain(&self.sensors)
            .chain(scanners)
            .chain(&self.sinks))
        .flatten()
        .chain(feedback)
        .collect()
    }
}

struct BeltWiring {
    run: Option<Port>,
    running: Option<String>,
    velocity: Option<String>,
    encoder: Option<String>,
    pulse: Option<String>,
}

/// The scene's addresses, in the layout's order.
pub(crate) struct Wiring {
    belts: Vec<BeltWiring>,
    sensors: Vec<Option<String>>,
    scanners: Vec<[Option<String>; 3]>,
    sinks: Vec<Option<String>>,
    feedback: Vec<String>,
}

fn signal(project: &Project, instance: &str, name: &str) -> Option<String> {
    let found = project.scene.instances.iter().find(|i| i.id == instance);
    found.and_then(|i| i.signals.get(name).cloned())
}

impl Wiring {
    pub fn of(project: &Project, layout: &Layout, field: Option<&Field>) -> Result<Self, String> {
        let at = |instance: &str, name: &str| signal(project, instance, name);
        let belts = (layout.belts.iter())
            .map(|b| {
                Ok(BeltWiring {
                    run: at(&b.id, "run").map(|a| Port::parse(&a)).transpose()?,
                    running: at(&b.id, "running"),
                    velocity: at(&b.id, "velocity"),
                    encoder: at(&b.id, "encoder"),
                    pulse: at(&b.id, "pulse"),
                })
            })
            .collect::<Result<_, String>>()?;
        let sensors = (layout.sensors.iter())
            .map(|s| {
                let name = match s.kind {
                    SensorKind::Diffuse { .. } | SensorKind::ThroughBeam => "detected",
                    SensorKind::Color { .. } => "color",
                    SensorKind::Laser { .. } => "distance",
                };
                at(&s.id, name)
            })
            .collect();
        let scanners = (layout.scanners.iter())
            .map(|s| ["read", "count", "code"].map(|name| at(&s.id, name)))
            .collect();
        let sinks = layout.sinks.iter().map(|s| at(&s.id, "count")).collect();
        let feedback = field.map_or_else(Vec::new, |f| f.feedback().to_vec());
        Ok(Self {
            belts,
            sensors,
            scanners,
            sinks,
            feedback,
        })
    }

    /// The ports `controller`'s program declares; an address it does not declare is not written, as Build warned.
    pub fn bind(&self, controller: &Controller) -> Ports {
        let declared = |address: &Option<String>| controller.declared_at(address.as_deref()?);
        Ports {
            belts: (self.belts.iter())
                .map(|b| BeltPorts {
                    run: b.run.clone(),
                    running: declared(&b.running),
                    velocity: declared(&b.velocity),
                    encoder: declared(&b.encoder),
                    pulse: declared(&b.pulse),
                })
                .collect(),
            sensors: self.sensors.iter().map(declared).collect(),
            scanners: (self.scanners.iter())
                .map(|s| s.each_ref().map(declared))
                .collect(),
            sinks: self.sinks.iter().map(declared).collect(),
            feedback: (self.feedback.iter())
                .filter_map(|a| Some((a.clone(), controller.declared_at(a)?)))
                .collect(),
        }
    }
}
