# 概要
二次元ユークリッド平面上における巡回セールスマン問題における焼きなまし法の実装で、近傍関数は2-optを使用し数種類の冷却スケジュールを実装してあります。
あくまで実験用のソースコードで、他人が使用することを前提に書かれていないので、可読性等にはかなり問題があると思われます。あと、コンパイルにすごく時間がかかります。

![example](./images/rat575_cool1_10sec.png)

# 使いかた
10回やって平均をとる
```
cargo run --release -- --time 10 --cooler c1 --iter 10
```

ビジュアライザを起動する
```
cargo run -- --time 10 --cooler c1 --iter 1 --vis
```
releaseビルドだと複数ウインドウが開けないバグがあります。

ログファイルに出力する
```
cargo run --release -- --time 3 --cooler c1 --iter 1 -D log.txt
```
1つのデータのみを1スレッドで実行します。ログファイルには上書きせず追記します。

# 著作権表示
[piston](https://github.com/PistonDevelopers/piston)、[slog](https://github.com/slog-rs/slog)、[clap](https://github.com/kbknapp/clap-rs)などのライブラリを使用しています。
また、データはTSPLIB[http://elib.zib.de/pub/mp-testdata/tsp/tsplib/tsplib.html]のものを使用しました。
このソフトウェア自体はUnlicenseのもとで公開します。
