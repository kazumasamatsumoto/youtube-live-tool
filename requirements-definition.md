# YouTube Live 配信ツール要件定義書

## 1. システム概要

### 1.1 目的

- YouTube Live 配信の統合管理
- 配信演出の効率化
- 視聴者とのインタラクション強化

### 1.2 対象プラットフォーム

- Windows (ARM/Snapdragon)

## 2. 機能要件

### 2.1 配信基本機能

- YouTube Live API との連携
- 配信開始/停止制御
- 配信品質設定
- 配信状態モニタリング

### 2.2 オーディオ管理

- BGM 再生機能
  - 複数曲登録
  - 音量調整
  - プレイリスト管理
- 効果音再生機能
  - ホットキー割り当て
  - 音量個別調整
- スーパーチャット効果音
  - 金額別自動再生
  - カスタム効果音設定

### 2.3 映像管理

- カメラ入力表示
  - 複数カメラ対応
  - 位置/サイズ調整
- 画面共有
  - モニター選択
  - ウィンドウ選択
  - 表示領域カスタマイズ

### 2.4 バナー管理

- カスタムバナー表示
  - 画像アップロード
  - テキスト編集
  - 表示位置/サイズ設定
- バナーローテーション

### 2.5 コメント管理

- コメント表示
- 自動読み上げ機能
  - VOICEVOX 連携（ずんだもん）
  - 読み上げ速度調整
  - NG ワード設定

## 3. 非機能要件

### 3.1 性能要件

- 配信遅延: 2 秒以内
- CPU 使用率: 30%以下
- メモリ使用量: 2GB 以下

### 3.2 信頼性要件

- 24 時間連続稼働対応
- 自動リカバリー機能

### 3.3 セキュリティ要件

- YouTube 認証情報の安全な管理
- ローカルデータの暗号化

## 4. 開発環境

- 言語: Rust
- UI フレームワーク: Druid
- ビルドツール: Cargo

## 5. 制約事項

- Windows (ARM)での動作保証
- インターネット接続必須
- VOICEVOX のローカルインストールが必要
