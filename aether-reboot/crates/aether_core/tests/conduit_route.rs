use aether_core::{
    Block,
    conduit_route::{
        Direction, MAX_STATES, MAX_WORK_PER_ADVANCE, Point, RouteError, Search, Socket, Space,
        Status,
    },
    fixtures,
};
use std::sync::Arc;

fn socket(point: Point, outward: Direction) -> Socket {
    Socket { point, outward }
}
fn finish(search: &mut Search, work: usize) -> Status {
    for _ in 0..100_000 {
        if search.advance(work) != &Status::Searching {
            return search.status().clone();
        }
    }
    panic!("bounded search failed to terminate");
}

#[test]
fn straight_route_is_exact_and_independent_of_work_slicing() {
    let space = Arc::new(Space::from_body(&fixtures::solid([1, 1, 1], Block::Wood)).unwrap());
    let start = socket(Point(-8, 8, 0), Direction::PosX);
    let end = socket(Point(8, 8, 0), Direction::NegX);
    let mut single = Search::new(space.clone(), start, end, MAX_STATES).unwrap();
    let mut sliced = Search::new(space, start, end, MAX_STATES).unwrap();
    assert_eq!(sliced.advance(0), &Status::Searching);
    assert_eq!(sliced.expanded_states(), 0);
    let result = finish(&mut single, 256);
    assert_eq!(result, finish(&mut sliced, 1));
    let Status::Found(route) = result else {
        panic!("direct route")
    };
    assert_eq!(route.points, vec![start.point, end.point]);
    assert_eq!(route.polyline_mm, 2000);
    assert_eq!(single.expanded_states(), sliced.expanded_states());
}

#[test]
fn route_avoids_real_hull_and_keeps_cardinal_socket_approaches() {
    let body = fixtures::solid([3, 3, 3], Block::Metal);
    let start = socket(Point(-10, 0, 4), Direction::PosX);
    let end = socket(Point(18, 0, 4), Direction::NegX);
    let mut search = Search::new(
        Arc::new(Space::from_body(&body).unwrap()),
        start,
        end,
        MAX_STATES,
    )
    .unwrap();
    let Status::Found(route) = finish(&mut search, 37) else {
        panic!("route around hull")
    };
    assert_eq!(route.points.first(), Some(&start.point));
    assert_eq!(route.points.last(), Some(&end.point));
    assert!(route.points[1].0 > start.point.0);
    assert!(route.points[route.points.len() - 2].0 < end.point.0);
    let mut length = 0;
    for pair in route.points.windows(2) {
        let a = aether_core::glam::Vec3::from_array(
            pair[0].millimetres().unwrap().map(|v| v as f32 / 1000.0),
        );
        let b = aether_core::glam::Vec3::from_array(
            pair[1].millimetres().unwrap().map(|v| v as f32 / 1000.0),
        );
        let delta = b - a;
        assert_eq!(delta.to_array().iter().filter(|v| **v != 0.0).count(), 1);
        assert!(delta.length() >= 0.5);
        length += (delta.length() * 1000.0).round() as u32;
        // Independent metre-space distance to every real closed voxel box.
        // Sample 5 mm along each full segment, not just route corners.
        let samples = (delta.length() / 0.005).ceil() as usize;
        for step in 0..=samples {
            let p = a.lerp(b, step as f32 / samples as f32);
            for (cell, _) in body.grid().iter() {
                let low = cell.center() - aether_core::glam::Vec3::splat(0.25);
                let high = cell.center() + aether_core::glam::Vec3::splat(0.25);
                assert!(p.distance(p.clamp(low, high)) > 0.28 - 0.00001);
            }
        }
    }
    assert_eq!(length, route.polyline_mm);
}

#[test]
fn equipment_footprints_are_obstacles_even_without_hull_blocks() {
    let body = fixtures::starter();
    let space = Space::from_body(&body).unwrap();
    // Real tank #3: supported at (-2,0,1), occupies the two cells above it.
    assert!(!space.is_free(Point(-8, 4, 4)));
    assert!(!space.is_free(Point(-8, 8, 4)));
    assert!(!space.is_free(Point(-8, 12, 4)));
    assert!(space.is_free(Point(-8, 13, 4)));
    assert!(!space.is_free(Point(0, 0, 0)));
}

#[test]
fn malformed_and_blocked_sockets_fail_before_search_allocation() {
    let space = Arc::new(Space::from_body(&fixtures::solid([1, 1, 1], Block::Wood)).unwrap());
    let start = socket(Point(-8, 8, 0), Direction::PosX);
    let end = socket(Point(8, 8, 0), Direction::NegX);
    assert!(matches!(
        Search::new(space.clone(), start, end, 0),
        Err(RouteError::Bounds)
    ));
    assert!(matches!(
        Search::new(space.clone(), start, end, usize::MAX),
        Err(RouteError::Bounds)
    ));
    assert!(matches!(
        Search::new(space.clone(), start, start, MAX_STATES),
        Err(RouteError::Bounds)
    ));
    assert!(matches!(
        Search::new(
            space.clone(),
            start,
            socket(Point(0, 0, 0), Direction::NegX),
            MAX_STATES
        ),
        Err(RouteError::Socket)
    ));
    let arbitrary = Point(i32::MAX, i32::MIN, 0);
    assert!(arbitrary.millimetres().is_none());
    assert!(!space.is_free(arbitrary));
    assert!(matches!(
        Search::new(space, start, socket(arbitrary, Direction::PosY), MAX_STATES),
        Err(RouteError::Socket)
    ));
}

#[test]
fn blocked_outward_stub_never_turns_inside_equipment() {
    let space = Arc::new(Space::from_body(&fixtures::solid([1, 1, 1], Block::Wood)).unwrap());
    let mut search = Search::new(
        space,
        socket(Point(-5, 0, 0), Direction::PosX),
        socket(Point(8, 8, 0), Direction::NegX),
        MAX_STATES,
    )
    .unwrap();
    assert_eq!(finish(&mut search, 256), Status::NoPath);
    assert_eq!(search.expanded_states(), 1);
}

#[test]
fn work_and_storage_limits_are_terminal_without_changing_the_hull() {
    let body = fixtures::solid([3, 3, 3], Block::Metal);
    let before = body.blueprint();
    let space = Arc::new(Space::from_body(&body).unwrap());
    let start = socket(Point(-10, 0, 4), Direction::PosX);
    let end = socket(Point(18, 0, 4), Direction::NegX);
    let mut search = Search::new(space.clone(), start, end, MAX_STATES).unwrap();
    search.advance(usize::MAX);
    assert!(search.expanded_states() <= MAX_WORK_PER_ADVANCE);
    let mut limited = Search::new(space, start, end, 16).unwrap();
    assert_eq!(finish(&mut limited, 256), Status::Limit);
    assert!(limited.stored_states() <= 16);
    let expanded = limited.expanded_states();
    assert_eq!(limited.advance(usize::MAX), &Status::Limit);
    assert_eq!(limited.expanded_states(), expanded);
    assert_eq!(body.blueprint(), before);
}
