# アーキテクチャ設計書

## ディレクトリ構成

```
druid-steaming-app/
├── assets/
│   └── fonts/                     # Noto Sans JPフォントファイル
├── config/
│   └── log4rs.yaml               # ログ設定ファイル
├── log/
│   ├── app.log                   # アプリケーションログ
│   └── camera.log                # カメラ関連ログ
├── src/
│   ├── app/
│   │   ├── main_window.rs        # メインウィンドウUI
│   │   ├── mod.rs               # アプリケーションモジュール定義
│   │   └── stream_window.rs      # ストリーミングウィンドウUI
│   ├── models/
│   │   ├── audio.rs             # 音声処理モデル
│   │   ├── banner.rs            # バナー表示モデル
│   │   ├── camera.rs            # カメラ制御モデル
│   │   ├── comment.rs           # コメント処理モデル
│   │   ├── gpu_processor.rs     # GPU処理モデル
│   │   ├── mod.rs              # モデルモジュール定義
│   │   ├── screen_capture.rs    # 画面キャプチャモデル
│   │   ├── stream.rs           # ストリーミング制御モデル
│   │   ├── video_config.rs     # 動画設定モデル
│   │   └── video_frame.rs      # 動画フレーム処理モデル
│   ├── shaders/
│   │   ├── color_convert.wgsl   # 色変換シェーダー
│   │   └── downscale.wgsl       # ダウンスケールシェーダー
│   ├── tabs/
│   │   ├── audio_tab.rs        # 音声設定タブUI
│   │   ├── banner_tab.rs       # バナー設定タブUI
│   │   ├── comment_tab.rs      # コメント設定タブUI
│   │   ├── mod.rs             # タブモジュール定義
│   │   ├── status_tab.rs      # ステータス表示タブUI
│   │   ├── stream_tab.rs      # ストリーム設定タブUI
│   │   └── video_tab.rs       # 動画設定タブUI
│   └── main.rs                 # アプリケーションエントリーポイント
```

## ファイルの責務

### アプリケーション層 (src/app/)
- **main_window.rs**: メインウィンドウのUIレイアウトと制御ロジック
- **stream_window.rs**: ストリーミングウィンドウのUIレイアウトと制御ロジック

### モデル層 (src/models/)
- **audio.rs**: 音声キャプチャと処理
- **banner.rs**: バナー表示の管理
- **camera.rs**: カメラデバイスの制御とキャプチャ
- **comment.rs**: コメント表示の管理
- **gpu_processor.rs**: GPU処理の制御（シェーダー実行など）
- **screen_capture.rs**: 画面キャプチャ機能
- **stream.rs**: ストリーミング配信の制御
- **video_config.rs**: 動画設定の管理
- **video_frame.rs**: フレームバッファの処理

### シェーダー (src/shaders/)
- **color_convert.wgsl**: BGRAからRGBAへの色空間変換
- **downscale.wgsl**: 画像のダウンスケール処理

### UI層 (src/tabs/)
- **audio_tab.rs**: 音声設定UI
- **banner_tab.rs**: バナー設定UI
- **comment_tab.rs**: コメント設定UI
- **status_tab.rs**: ステータス表示UI
- **stream_tab.rs**: ストリーム設定UI
- **video_tab.rs**: 動画設定UI

### 設定・ログ
- **config/log4rs.yaml**: ログ出力の設定
- **log/**: アプリケーションとカメラのログファイル
