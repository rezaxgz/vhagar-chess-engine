use std::time::Instant;

use crate::core::{board::Board, perft::perft};
#[derive(Clone, Copy)]
struct PerftTestCase<'a> {
    fen: &'a str,
    results: [usize; 8],
}
const TEST_CASES: [PerftTestCase; 25] = [
    PerftTestCase {
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        results: [1, 20, 400, 8902, 197281, 4865609, 119060324, 3195901860],
    },
    PerftTestCase {
        fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        results: [1, 48, 2039, 97862, 4085603, 193690690, 8031647685, 0],
    },
    PerftTestCase {
        fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        results: [1, 14, 191, 2812, 43238, 674624, 11030083, 178633661],
    },
    PerftTestCase {
        fen: "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        results: [1, 6, 264, 9467, 422333, 15833292, 706045033, 0],
    },
    PerftTestCase {
        fen: "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        results: [1, 44, 1486, 62379, 2103487, 89941194, 0, 0],
    },
    PerftTestCase {
        fen: "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        results: [1, 46, 2079, 89890, 3894594, 164075551, 6923051137, 0],
    },
    PerftTestCase {
        fen: "8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1",
        results: [1, 15, 126, 1928, 13931, 206379, 1440467, 0],
    },
    PerftTestCase {
        fen: "8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1",
        results: [1, 15, 126, 1928, 13931, 206379, 1440467, 0],
    },
    PerftTestCase {
        fen: "5k2/8/8/8/8/8/8/4K2R w K - 0 1",
        results: [1, 15, 66, 1198, 6399, 120330, 661072, 0],
    },
    PerftTestCase {
        fen: "r3k3/8/8/8/8/8/8/3K4 b q - 0 1",
        results: [1, 16, 71, 1286, 7418, 141077, 803711, 0],
    },
    PerftTestCase {
        fen: "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1",
        results: [1, 26, 1141, 27826, 1274206, 31912360, 0, 0],
    },
    PerftTestCase {
        fen: "r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1",
        results: [1, 26, 1141, 27826, 1274206, 31912360, 0, 0],
    },
    PerftTestCase {
        fen: "r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1",
        results: [1, 44, 1494, 50509, 1720476, 58773923, 0, 0],
    },
    PerftTestCase {
        fen: "r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1",
        results: [1, 44, 1494, 50509, 1720476, 58773923, 0, 0],
    },
    PerftTestCase {
        fen: "2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1",
        results: [1, 11, 133, 1442, 19174, 266199, 3821001, 0],
    },
    PerftTestCase {
        fen: "3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1",
        results: [1, 11, 133, 1442, 19174, 266199, 3821001, 0],
    },
    PerftTestCase {
        fen: "8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1",
        results: [1, 29, 165, 5160, 31961, 1004658, 6334638, 0],
    },
    PerftTestCase {
        fen: "5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1",
        results: [1, 29, 165, 5160, 31961, 1004658, 6334638, 0],
    },
    PerftTestCase {
        fen: "4k3/1P6/8/8/8/8/K7/8 w - - 0 1",
        results: [1, 9, 40, 472, 2661, 38983, 217342, 0],
    },
    PerftTestCase {
        fen: "8/k7/8/8/8/8/1p6/4K3 b - - 0 1",
        results: [1, 9, 40, 472, 2661, 38983, 217342, 0],
    },
    PerftTestCase {
        fen: "8/P1k5/K7/8/8/8/8/8 w - - 0 1",
        results: [1, 6, 27, 273, 1329, 18135, 92683, 0],
    },
    PerftTestCase {
        fen: "8/8/8/8/8/k7/p1K5/8 b - - 0 1",
        results: [1, 6, 27, 273, 1329, 18135, 92683, 0],
    },
    PerftTestCase {
        fen: "K1k5/8/P7/8/8/8/8/8 w - - 0 1",
        results: [1, 2, 6, 13, 63, 382, 2217, 0],
    },
    PerftTestCase {
        fen: "8/8/8/8/8/p7/8/k1K5 b - - 0 1",
        results: [1, 2, 6, 13, 63, 382, 2217, 0],
    },
    PerftTestCase {
        fen: "8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1",
        results: [1, 37, 183, 6559, 23527, 811573, 3114998, 0],
    },
];
static mut NODES: usize = 0;
pub fn test_perft() {
    let start = Instant::now();
    let mut result = [true; TEST_CASES.len()];
    let mut passed = true;
    for i in 0..TEST_CASES.len() {
        println!("position fen {}", TEST_CASES[i].fen);
        let res = test_case(i);
        passed &= res;
        if !res {
            result[i] = false;
        }
    }
    if passed {
        let mut total_nodes = 0;
        for i in 0..TEST_CASES.len(){
            for j in 0..TEST_CASES[i].results.len(){
                total_nodes += TEST_CASES[i].results[j];
            }
        }
        println!("perft test completed successfully in {:?}", start.elapsed());
        println!("total nodes: {}", total_nodes);
        println!("nodes per second: {}", (total_nodes * 1000) / (start.elapsed().as_millis() as usize));

    } else {
        for i in 0..result.len() {
            if !result[i] {
                println!("perft failed test {} fen {}", i, TEST_CASES[i].fen);
            }
        }
    }
}
pub fn test_case(i: usize) -> bool {
    let mut passed = true;
    unsafe {
        NODES = 0;
    }
    let start = Instant::now();
    let case = TEST_CASES[i];
    let board = Board::from_fen(case.fen);
    for depth in 0..case.results.len() {
        if case.results[depth] != 0 {
            let mut moves = Vec::with_capacity(10);
            let res = perft(&board, depth, &mut moves);
            unsafe {
                NODES += NODES + res;
            }
            if res != case.results[depth] {
                passed = false;
            }
        }
    }

    unsafe {
        let nps = ((NODES as f64) / (start.elapsed().as_micros() as f64)) * 1000_000.0;
        println!(
            "perft test case {} in {:?} nodes {} nps {}",
            i + 1,
            start.elapsed(),
            NODES,
            nps.floor()
        );
    }
    return passed;
}
