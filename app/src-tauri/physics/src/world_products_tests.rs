//! Products and their sources (spec 2026-11-11-sensors-products-design.md, acceptance 2). The reference sequences are
//! OIP's `src/core/rng.ts`, run in Node by hand (2026-11-11): the first five values of each seed, times 2³².

use super::super::tests::{belt, run, source};
use super::*;
use crate::layout::{Layout, Source};

fn draws(seed: u32) -> Vec<u32> {
    let mut rng = Mulberry32::new(seed);
    (0..5)
        .map(|_| (rng.next() * 4_294_967_296.0) as u32)
        .collect()
}

#[test]
fn mulberry32_gives_oips_sequences() {
    assert_eq!(
        draws(1),
        [2693262067, 11749833, 2265367787, 4213581821, 4159151403]
    );
    assert_eq!(
        draws(0),
        [1144304738, 1416247, 958946056, 627933444, 2007157716]
    );
    assert_eq!(
        draws(-7_i32 as u32),
        [1860010037, 1397564179, 2337619704, 2062400319, 209248909]
    );
    assert_eq!(
        draws(20_260_919),
        [3623576882, 1678360307, 391502442, 3711438689, 126556203]
    );
}

fn products(colors: usize, codes: usize) -> Products {
    let palette = [[255, 0, 0], [0, 255, 0], [0, 0, 255]];
    Products {
        colors: palette[..colors].to_vec(),
        codes: [10, 20, 30][..codes].to_vec(),
        ..Products::carton()
    }
}

fn drawn(p: &Products, n: usize) -> Vec<Drawn> {
    let mut state = SourceState::new(0, p.seed);
    (0..n).map(|_| state.draw(p)).collect()
}

#[test]
fn the_lists_are_walked_in_step_and_each_cycles() {
    let picked: Vec<_> = drawn(&products(3, 2), 6)
        .into_iter()
        .map(|d| (d.color.unwrap()[0..3].to_vec(), d.code.unwrap()))
        .collect();
    let (r, g, b) = (vec![255, 0, 0], vec![0, 255, 0], vec![0, 0, 255]);
    assert_eq!(
        picked,
        [
            (r.clone(), 10),
            (g.clone(), 20),
            (b.clone(), 10),
            (r, 20),
            (g, 10),
            (b, 20)
        ]
    );
}

#[test]
fn without_lists_a_product_is_unpainted_without_a_code_and_the_size_is_the_set_one() {
    let p = Products {
        size: Vector::new(0.6, 0.4, 0.5),
        ..Products::carton()
    };
    let d = &drawn(&p, 1)[0];
    assert_eq!(
        d,
        &Drawn {
            // Length along x, height up y, width across z.
            half: Vector::new(0.3, 0.25, 0.2),
            color: None,
            code: None,
        }
    );
}

#[test]
fn in_random_order_one_draw_picks_both_the_colour_and_the_code() {
    let p = Products {
        random_order: true,
        seed: 1,
        ..products(3, 2)
    };
    let mut rng = Mulberry32::new(1);
    for d in drawn(&p, 5) {
        let r = rng.next();
        assert_eq!(d.color, Some(p.colors[(r * 3.0) as usize]));
        assert_eq!(d.code, Some(p.codes[(r * 2.0) as usize]));
    }
}

#[test]
fn a_random_size_is_drawn_after_the_colour_as_length_height_width_in_oips_range() {
    let p = Products {
        random_size: true,
        random_order: true,
        seed: 20_260_919,
        ..products(3, 0)
    };
    let mut rng = Mulberry32::new(20_260_919);
    for d in drawn(&p, 20) {
        let r = rng.next();
        assert_eq!(d.color, Some(p.colors[(r * 3.0) as usize]));
        assert_eq!(d.code, None);
        let [length, height, width] = [(0.4, 0.8), (0.3, 0.5), (0.3, 0.5)]
            .map(|(lo, hi): (f64, f64)| (lo + rng.next() * (hi - lo)) as Real / 2.0);
        assert_eq!(d.half, Vector::new(length, height, width));
        assert!(
            (0.2..0.4).contains(&d.half.x) && (0.15..0.25).contains(&d.half.y),
            "{d:?}"
        );
        assert!((0.15..0.25).contains(&d.half.z), "{d:?}");
    }
    let walked = Products {
        random_order: false,
        ..p.clone()
    };
    let mut rng = Mulberry32::new(20_260_919);
    let first = &drawn(&walked, 1)[0];
    assert_eq!(first.color, Some(p.colors[0]), "a walk draws nothing");
    assert_eq!(first.half.x, (0.4 + rng.next() * 0.4) as Real / 2.0);
}

/// One source on a 6 m belt making `products`, every 2 s.
fn line(products: Products, speed: Real) -> World {
    let mut w = World::new(Layout {
        belts: vec![belt("a", 0.0, 6.0, speed, true)],
        sources: vec![Source {
            products,
            ..source(0, 1_000_000, 2_000_000_000, 0)
        }],
        ..Layout::default()
    });
    w.set_belt_running(0, true);
    w
}

#[test]
fn a_product_has_its_size_mass_and_colour_and_rests_on_the_belt() {
    let p = Products {
        size: Vector::new(0.6, 0.4, 0.4),
        mass: 10.0,
        ..products(2, 1)
    };
    let mut w = line(p, 0.5);
    run(&mut w, 1000);
    let body = &w.world.bodies[w.products[0].body];
    assert!((body.mass() - 10.0).abs() < 1e-4, "{}", body.mass());
    let c = &w.cartons()[0];
    assert_eq!((c.size, c.color), ([0.6, 0.4, 0.4], Some([255, 0, 0])));
    assert!(
        (c.position[1] - 1.0).abs() < 0.002,
        "rests on the 0.8 m belt: {:?}",
        c.position
    );
    assert_eq!(w.products[0].code, Some(10));
    run(&mut w, 2000);
    assert_eq!(w.cartons()[1].color, Some([0, 255, 0]));
}

#[test]
fn a_product_that_waits_for_a_free_entry_keeps_what_it_was_drawn_with() {
    let p = Products {
        random_size: true,
        seed: 7,
        ..products(3, 3)
    };

    // The belt stands still for 5 s: the first product holds the entry, the second waits through two intervals.
    let mut w = line(p.clone(), 0.5);
    w.set_belt_running(0, false);
    run(&mut w, 5000);
    assert_eq!(w.created(), 1);
    w.set_belt_running(0, true);
    run(&mut w, 6000);
    let made: Vec<_> = (w.products.iter())
        .map(|p| (p.half, p.color, p.code))
        .collect();
    let want: Vec<_> = (drawn(&p, made.len()).iter())
        .map(|d| (d.half, d.color, d.code))
        .collect();
    assert!(made.len() >= 3, "{made:?}");
    assert_eq!(made, want, "none redrawn, none skipped");
}

#[test]
fn the_same_seed_makes_the_same_line_and_another_seed_another() {
    let p = Products {
        random_size: true,
        random_order: true,
        ..products(3, 3)
    };
    let fingerprint = |seed| {
        let mut w = line(Products { seed, ..p.clone() }, 0.5);
        run(&mut w, 7000);
        w.fingerprint()
    };
    assert_eq!(fingerprint(3), fingerprint(3));
    assert_ne!(fingerprint(3), fingerprint(4));
}

#[test]
fn a_products_reach_follows_how_it_lies() {
    let mut w = line(
        Products {
            size: Vector::new(0.6, 0.4, 0.2),
            ..Products::carton()
        },
        0.0,
    );
    run(&mut w, 2);
    let product = &w.products[0];
    assert!((w.reach(product, Vector::X) - 0.3).abs() < 1e-5);
    // 0.6 long, 0.4 wide, 0.2 high.
    assert!((w.reach(product, Vector::Y) - 0.1).abs() < 1e-5);
    assert!((w.reach(product, Vector::Z) - 0.2).abs() < 1e-5);
    let body = product.body;
    let yaw = Rotation::from_rotation_y(std::f32::consts::FRAC_PI_4);
    w.world.bodies[body].set_rotation(yaw, true);
    let product = &w.products[0];
    let diagonal = (0.3 + 0.2) * std::f32::consts::FRAC_1_SQRT_2;
    assert!((w.reach(product, Vector::X) - diagonal).abs() < 1e-5);
    assert!((w.reach(product, Vector::Z) - diagonal).abs() < 1e-5);
    assert!((w.reach(product, -Vector::Y) - 0.1).abs() < 1e-5);
}

#[test]
fn a_source_reads_its_products_and_one_pinned_before_them_makes_the_carton() {
    use twinova_project::model::{Item, ParamValue};
    let mut project = super::super::tests::fixture("sort_line");
    let products_of = |p: &twinova_project::Project| {
        crate::layout::layout(p).unwrap().sources[0]
            .products
            .clone()
    };
    assert_eq!(products_of(&project), Products::carton());
    let source = (project.scene.instances.iter_mut())
        .find(|i| i.id == "source")
        .unwrap();
    let text = |t: &str| Item::Text(t.into());
    for (name, value) in [
        ("length", ParamValue::Real(0.6)),
        ("height", ParamValue::Real(0.5)),
        ("mass", ParamValue::Real(10.0)),
        ("random_size", ParamValue::Bool(true)),
        (
            "colors",
            ParamValue::List(vec![text("#ff0000"), text("#00ff7f")]),
        ),
        ("codes", ParamValue::List(vec![Item::Int(3), Item::Int(-1)])),
        ("random_order", ParamValue::Bool(true)),
        ("seed", ParamValue::Int(-7)),
    ] {
        source.parameters.insert(name.into(), value);
    }
    assert_eq!(
        products_of(&project),
        Products {
            size: Vector::new(0.6, 0.3, 0.5),
            mass: 10.0,
            random_size: true,
            colors: vec![[255, 0, 0], [0, 255, 127]],
            codes: vec![3, -1],
            random_order: true,
            seed: -7_i32 as u32,
        }
    );
    // A source definition pinned before 4.4d has none of these parameters.
    let source = (project.scene.instances.iter_mut())
        .find(|i| i.id == "source")
        .unwrap();
    source.parameters.clear();
    let definition = project.definitions.get_mut("twinova.source@1").unwrap();
    (definition.parameters)
        .retain(|p| ["interval", "first_arrival", "max_count"].contains(&p.name.as_str()));
    assert_eq!(products_of(&project), Products::carton());
}

/// OIP's rounding: 2 cm, or 0.4 of the smallest half extent when that is less (`makeBoxShape`).
#[test]
fn a_products_edges_are_rounded_by_its_smallest_half_extent() {
    for (size, rounding) in [(0.3, 0.02), (0.06, 0.012)] {
        let mut w = line(
            Products {
                size: Vector::splat(size),
                ..Products::carton()
            },
            0.0,
        );
        run(&mut w, 2);
        let body = &w.world.bodies[w.products[0].body];
        let collider = &w.world.colliders[body.colliders()[0]];
        let round = collider.shape().as_round_cuboid().expect("a rounded box");
        assert!(
            (round.border_radius - rounding).abs() < 1e-6,
            "{size}: {}",
            round.border_radius
        );
        let full = round.inner_shape.half_extents + Vector::splat(round.border_radius);
        assert!((full - Vector::splat(size / 2.0)).abs().max_element() < 1e-6);
    }
}
