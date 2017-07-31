#[macro_use]
extern crate clap;
extern crate piston_window;
extern crate rand;
#[macro_use]
extern crate slog;
extern crate sloggers;
#[macro_use]
extern crate lazy_static;
use piston_window::*;
mod scanner;
mod point;
use scanner::Scanner;
use std::fs::File;
use point::Point;
use std::time::*;
use clap::ArgMatches;
use sloggers::types::Severity;
use sloggers::Build;
type Real = f64;
const INF: Real = 1e18;
#[inline]
fn loge(x: Real) -> Real {
    x.log((1.0 as Real).exp())
}

// program configurations
lazy_static!{
    static ref MATCHES: ArgMatches<'static> =
        clap_app!(sa_test =>
                  (version: "1.0")
                  (author: "kngwyu")
                  (about: "Test of simulated annealing")
                  (@arg TIME: -T --time +takes_value "Sets execution time(default 5)")
                  (@arg COOLER: -C --cooler +takes_value "Sets cooler type(default climb)")
                  (@arg ITER: -I --iter +takes_value "Sets iteration number(default 1)")
                  (@arg FNAME: -D --debug +takes_value "Use debug mode(only 1 thread runs and create Log file<FNAME>)")
                  (@arg VIS: -V --vis "Use Visualize(defualt false)")
        ).get_matches();
    static ref LOGGER: slog::Logger = match MATCHES.value_of("FNAME") {
        Some(s) => sloggers::file::FileLoggerBuilder::new(s).level(Severity::Debug).build(),
        None => sloggers::null::NullLoggerBuilder{}.build(),
    }.ok().unwrap();
}
// x座標が近いのからてきとーに選ぶ
// 最悪O(N^2)だけどならしはO(NlogN)くらいだと思う
fn group_ord_fast(__points: &Vec<Point>) -> Vec<usize> {
    let mut points = __points.clone();
    points.sort();
    let n = points.len();
    let mut res = vec![0usize; n];
    let mut used = vec![false; n];
    let mut cur = 0;
    for i in 0..n {
        used[cur] = true;
        res[i] = cur;
        if i == n - 1 {
            break;
        }
        let mut nxt_node = 0;
        for neib in 1..n {
            let j = (i + neib) % n;
            if !used[j] {
                nxt_node = j;
                break;
            }
            let j = (i + n - neib) % n;
            if !used[j] {
                nxt_node = j;
                break;
            }
        }
        cur = nxt_node;
    }
    res
}
// 貪欲法で0番ノードから順に経路作成 O(V^2)
fn group_ord_greedy(points: &Vec<Point>) -> Vec<usize> {
    let n = points.len();
    if n > 2000 {
        return group_ord_fast(points);
    }
    let mut res = vec![0usize; n];
    let mut used = vec![false; n];
    let mut cur = 0;
    for i in 0..n {
        used[cur] = true;
        res[i] = cur;
        if i == n - 1 {
            break;
        }
        let mut min_dist = INF;
        let mut nxt_node = 0;
        for j in 0..n {
            if used[j] {
                continue;
            }
            let d = point::dist(points[cur], points[j]);
            if d < min_dist {
                min_dist = d;
                nxt_node = j;
            }
        }
        cur = nxt_node;
    }
    res
}

// a0 - a1 - ... - b1 - b2
// => a0 - b1 - .... - a1 - b2
// スコアの増加分を返す
fn calc_2opt(ord: &Vec<usize>, points: &Vec<Point>, ai: usize, bi: usize) -> Real {
    use point::dist;
    let n = ord.len();
    let (a, b, a1, b1) = (ord[ai], ord[bi], ord[(ai + 1) % n], ord[(bi + 1) % n]);
    let dist_cur = dist(points[a], points[a1]) + dist(points[b], points[b1]);
    let dist_nxt = dist(points[a], points[b]) + dist(points[a1], points[b1]);
    dist_nxt - dist_cur
}
#[derive(Debug, Clone)]
struct SaState {
    score: Real,
    ord: Vec<usize>,
    m1: usize, // スコアが減少した場合の遷移
    m2: usize, // スコアが増加した場合の遷移回数
    delta_c_sum: Real, // m2での遷移のdelta_cの合計
    cost_sum: Real, // Markov Chainに含まれる遷移状態のコストの合計
    cost_sq_sum: Real, // Markov Chainに含まれる遷移状態のコストの二乗和
}
impl SaState {
    fn change_2opt(&mut self, a: usize, b: usize, score_diff: Real) {
        let n = self.ord.len();
        let mut i = (a + 1) % n;
        let mut rev_i = b;
        let mut ord = self.ord.clone();
        while rev_i != a {
            ord[i] = self.ord[rev_i];
            rev_i = (rev_i + n - 1) % n;
            i = (i + 1) % n;
        }
        self.ord = ord;
        self.score += score_diff;
        self.cost_sum += self.score;
        self.cost_sq_sum += self.score * self.score;
    }
}
trait Cooler {
    fn get_prob(&mut self, sa: &SaState, delta_c: Real) -> Real;
    fn decrement(&mut self, sa: &SaState);
    fn print_info(&self);
}
// 山のぼり
struct Climb {}
impl Climb {
    fn new() -> Climb {
        Climb {}
    }
}
impl Cooler for Climb {
    fn get_prob(&mut self, sa: &SaState, delta_c: Real) -> Real {
        0.0
    }
    fn decrement(&mut self, sa: &SaState) {}
    fn print_info(&self) {}
}
// theoretical and computational aspects of simulated annealing p45の1番
struct Cooler1 {
    x: Real,
    c: Real,
    count: usize,
    delta: Real,
}
impl Cooler1 {
    fn new() -> Cooler1 {
        Cooler1 {
            x: 0.95,
            c: 0.0,
            count: 0,
            delta: 0.1,
        }
    }
}
impl Cooler for Cooler1 {
    fn get_prob(&mut self, sa: &SaState, delta_c: Real) -> Real {
        if sa.m2 == 0 {
            self.x
        } else if self.count < 100 {
            self.x
        } else {
            let p = (-delta_c / self.c).exp();
            p
        }
    }
    fn decrement(&mut self, sa: &SaState) {
        self.count += 1;
        if self.count < 100 {
            if self.count == 1 {
                // むりやりCSVにするために変なことをしている
                info!(LOGGER, ",count, c,");
            }
            if sa.m1 == 0 || sa.m2 == 0 {
                return;
            }
            let (m1, m2) = (sa.m1 as Real, sa.m2 as Real);
            let delta_c_ave = sa.delta_c_sum / m2;
            self.c = delta_c_ave / loge(m2 / (m2 * self.x - m1 * (1.0 - self.x)));
            self.x = (m1 + m2 * (-delta_c_ave / self.c).exp()) / (m1 + m2);
        } else {
            let m0 = (sa.m1 + sa.m2) as Real;
            let mean = sa.cost_sum / m0;
            let var = sa.cost_sq_sum / m0 - mean * mean;
            self.c = self.c / (1.0 + self.c * loge(1.0 + self.delta) / (3.0 * var.sqrt()));
            if self.count % 300 == 0 {
                info!(LOGGER, ",{}, {},", self.count, self.c);
            }
        }
    }
    fn print_info(&self) {
        println!("Cooler1 count: {}, c: {}", self.count, self.c)
    }
}
// theoretical and computational aspects of simulated annealing p45の2番
struct Cooler2 {
    x: Real,
    c: Real,
    count: usize,
    alpha: Real,
}

impl Cooler2 {
    fn new() -> Cooler2 {
        Cooler2 {
            x: 0.95,
            c: 0.0,
            count: 0,
            alpha: 0.999995, // 手動で調整してください
        }
    }
}
impl Cooler for Cooler2 {
    fn get_prob(&mut self, sa: &SaState, delta_c: Real) -> Real {
        self.count += 1;
        if sa.m2 == 0 {
            self.x
        } else if self.count < 100 {
            self.x
        } else {
            let p = (-delta_c / self.c).exp();
            p
        }
    }
    fn decrement(&mut self, sa: &SaState) {
        if self.count < 100 {
            if sa.m1 == 0 || sa.m2 == 0 {
                return;
            }
            let (m1, m2) = (sa.m1 as Real, sa.m2 as Real);
            let delta_c_ave = sa.delta_c_sum / m2;
            self.c = delta_c_ave / loge(m2 / (m2 * self.x - m1 * (1.0 - self.x)));
            self.x = (m1 + m2 * (-delta_c_ave / self.c).exp()) / (m1 + m2);
        } else {
            self.c *= self.alpha;
        }
    }
    fn print_info(&self) {
        println!("Cooler2: count: {}, c: {}", self.count, self.c)
    }
}
struct Cooler3 {
    t0: Real,
    n: Real,
    alpha: Real,
    beta: Real,
}
impl Cooler3 {
    fn new() -> Cooler3 {
        Cooler3 {
            t0: 30000.0,
            n: 50000.0 * 3.0, // 一秒に五万回遷移すると仮定
            alpha: 1.0,
            beta: 4.0,
        }
    }
}

impl Cooler for Cooler3 {
    fn get_prob(&mut self, sa: &SaState, delta_c: Real) -> Real {
        let progress = ((sa.m1 + sa.m2) as Real + 0.5) / self.n;
        let remain = 1.0 - progress;
        let t = self.t0 * remain.powf(self.alpha) * (-progress * self.beta).exp2();
        let p = (-delta_c / t).exp();
        p
    }
    fn decrement(&mut self, sa: &SaState) {}
    fn print_info(&self) {}
}

// 焼きなまし法のコード
fn annealing<C: Cooler>(
    points: &Vec<Point>,
    time_limit: Duration,
    mut cooler: C,
) -> (SaState, SaState) {
    use rand::{self, Rng};
    let mut rng = rand::thread_rng();
    let mut rng2 = rand::thread_rng();
    let mut ratio = || {
        let r = rng2.next_u64() as f64;
        r / u64::max_value() as f64
    };
    let init_ord = group_ord_greedy(points);
    let n = init_ord.len();
    let init_score = (0..n).fold(0f64, |acc, i| {
        acc + point::dist(points[init_ord[i]], points[init_ord[(i + 1) % n]])
    });
    let mut sa_state = SaState {
        score: init_score,
        ord: init_ord,
        m1: 0,
        m2: 0,
        delta_c_sum: 0.0,
        cost_sum: init_score,
        cost_sq_sum: init_score * init_score,
    };
    let mut counter = 0;
    let start_time = SystemTime::now();
    let mut best_state = sa_state.clone();
    // ここで冷却方法を指定する
    loop {
        counter += 1;
        if (counter & 0b111111) == 0b111111 {
            match start_time.elapsed() {
                Ok(elapse) => {
                    if elapse > time_limit {
                        break;
                    }
                }
                Err(e) => panic!("Error in SysTime::elapsed {}", e),
            }
        }
        let node_a: usize = rng.gen_range(0, n);
        let node_b: usize = rng.gen_range(0, n);
        if node_a == node_b {
            continue;
        }
        let delta_c = calc_2opt(&sa_state.ord, points, node_a, node_b);
        // 遷移確率の計算
        let do_transit = {
            if delta_c <= 0.0 {
                sa_state.m1 += 1;
                true
            } else if ratio() <= cooler.get_prob(&sa_state, delta_c) {
                sa_state.m2 += 1;
                sa_state.delta_c_sum += delta_c;
                true
            } else {
                false
            }
        };
        if do_transit {
            sa_state.change_2opt(node_a, node_b, delta_c);
            cooler.decrement(&sa_state);
        }
        if sa_state.score < best_state.score {
            best_state = sa_state.clone();
        }
    }
    cooler.print_info();
    (best_state, sa_state)
}

fn main() {
    use std::thread;
    use std::sync::Arc;
    let time = match MATCHES.value_of("TIME").unwrap_or("5").parse::<u64>() {
        Ok(t) => t,
        Err(_) => panic!("usage: --time 10"),
    };
    let time_limit = Duration::from_secs(10);
    let coolers = Arc::new(MATCHES.value_of("COOLER").unwrap_or("climb").to_owned());
    let vis = MATCHES.is_present("VIS");
    let loop_num = match MATCHES.value_of("ITER").unwrap_or("1").parse::<usize>() {
        Ok(t) => t,
        Err(_) => panic!("usage: --iter 10"),
    };
    println!(
        "time {}, cooler: {}, vis: {}, iterate: {}",
        time,
        coolers,
        vis,
        loop_num
    );
    let data_set = ["bier127.tsp", "a280.tsp", "pr299.tsp", "rat575.tsp"];
    // デバッグ用
    let climb_score = [
        122488.65442435951,
        2788.7170621456303,
        52136.86539125871,
        7304.058724311905,
    ];
    let mut children = Vec::new();
    let data_num = if MATCHES.is_present("FNAME") {
        println!("DEBUG MODE");
        1
    } else {
        data_set.len()
    };
    for i in 0..data_num {
        let cooler = Arc::clone(&coolers);
        children.push(thread::spawn(move || {
            let s = data_set[i];
            let p = get_tsp_data(s);
            let mut sum = 0.0;
            let mut sqsum = 0.0;
            for _ in 0..loop_num {
                let (best_state, final_state) = match &**cooler {
                    "climb" => annealing(&p, time_limit, Climb::new()),
                    "c1" => annealing(&p, time_limit, Cooler1::new()),
                    "c2" => annealing(&p, time_limit, Cooler2::new()),
                    "c3" => annealing(&p, time_limit, Cooler3::new()),
                    _ => annealing(&p, time_limit, Climb::new()),
                };
                // println!(
                //     "{} best score: {}(m1: {}, m2: {}) final score: {}(m1: {}, m2: {})",
                //     s,
                //     best_state.score,
                //     best_state.m1,
                //     best_state.m2,
                //     final_state.score,
                //     final_state.m1,
                //     final_state.m2
                // );
                sum += best_state.score;
                sqsum += best_state.score * best_state.score;
                if vis {
                    final_state.draw_tsp(&p, s);
                }
            }
            let n = loop_num as Real;
            let var = {
                let tmp = sum / n;
                sqsum / n - tmp * tmp
            };
            let imp = (climb_score[i] - (sum / n)) / climb_score[i];
            println!(
                "{} score: {} improve: {}, variance: {}",
                s,
                sum / n,
                imp,
                var
            );
        }));
        // 集計モード
    }
    for child in children {
        let _ = child.join();
    }
}

// 二次元ユークリッド座標上でのTSPを解く
// TSBLIBフォーマットのデータを読む
const DATA_DIR: &'static str = "./data/";
fn get_tsp_data(fname: &str) -> Vec<Point> {
    let path = &*format!("{}{}", DATA_DIR, fname);
    let tspfile = match File::open(path) {
        Ok(f) => f,
        Err(_) => panic!("coulnd't open file: {}", path),
    };
    let mut sc = Scanner::new(tspfile);
    let node_num = {
        let mut res = 0;
        while let Some(s) = sc.next::<String>() {
            match &*s {
                "DIMENSION" => {
                    let _ = sc.next::<String>();
                    res = sc.next::<usize>().unwrap();
                }
                "EDGE_WEIGHT_TYPE" => {
                    let _ = sc.next::<String>();
                    if let Some(t) = sc.next::<String>() {
                        if &*t != "EUC_2D" {
                            panic!("Error: Dist Type is not EUC_2D, but {}", &*t);
                        }
                    }
                }
                "NODE_COORD_SECTION" => break,
                _ => {}
            }
        }
        res
    };
    let mut res = vec![Point::new(0.0, 0.0); node_num];
    for p in &mut res {
        let _: usize = sc.ne();
        let (x, y): (Real, Real) = (sc.ne(), sc.ne());
        *p = Point::new(x, y);
    }
    res
}

// Visualiizer Code
impl SaState {
    fn draw_tsp(&self, points: &Vec<Point>, fname: &str) {
        const WINDOW_SIZE: u32 = 600;
        let title = &*format!("{} score: {}", fname, self.score);
        let (unit_dist, offsetx, offsety) = {
            let xmax = points.iter().fold(0f64, |acc, cd| acc.max(cd.x));
            let ymax = points.iter().fold(0f64, |acc, cd| acc.max(cd.y));
            let xmin = points.iter().fold(0f64, |acc, cd| acc.min(cd.x));
            let ymin = points.iter().fold(0f64, |acc, cd| acc.min(cd.y));
            (
                WINDOW_SIZE as f64 / (xmax - xmin).max(ymax - ymin),
                -xmin,
                -ymin,
            )
        };
        let p2cd = |p: Point| {
            (
                (p.x + offsetx) * unit_dist + 1.0,
                (p.y + offsety) * unit_dist + 1.0,
            )
        };
        let mut window: PistonWindow =
            WindowSettings::new(title, [WINDOW_SIZE + 5, WINDOW_SIZE + 5])
                .exit_on_esc(true)
                .opengl(OpenGL::V3_2)
                .build()
                .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));
        window.set_lazy(true);
        let mut init = false;
        while let Some(e) = window.next() {
            if let Some(button) = e.press_args() {
                if button == Button::Keyboard(Key::X) {
                    break;
                }
            }
            if init {
                continue;
            }
            window.draw_2d(&e, |c, g| {
                clear([1.0; 4], g);
                let mut cur = self.ord[0];
                for i in 0..self.ord.len() {
                    let nxt = self.ord[(i + 1) % self.ord.len()];
                    let cd1 = p2cd(points[cur]);
                    let cd2 = p2cd(points[nxt]);
                    line(
                        [0.0, 0.0, 0.0, 1.0],
                        (unit_dist / 5.0).max(0.6),
                        [cd1.0, cd1.1, cd2.0, cd2.1],
                        c.transform,
                        g,
                    );
                    ellipse(
                        [1.0, 0.0, 0.0, 1.0],
                        [cd2.0, cd2.1, unit_dist, unit_dist],
                        c.transform,
                        g,
                    );
                    cur = nxt;
                }
            });
            init = true;
        }
    }
}
