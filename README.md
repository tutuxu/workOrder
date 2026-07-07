# workOrder

涓汉宸ヤ綔浜嬮」杩借釜妗岄潰搴旂敤銆傚熀浜?Tauri 2 + Vue 3 + Rust锛屾暟鎹繚瀛樺湪鏈湴 SQLite銆?
## 鑾峰彇鍙繍琛岀増鏈紙鏈€缁堢敤鎴凤級

鑻ヤ綘**鍙渶瑕佽繍琛岀▼搴忋€佷笉闇€瑕佹簮鐮?*锛岃鍏嬮殕鍙戝竷鍒嗘敮锛?
```bash
git clone -b workOrder-release <浠撳簱鍦板潃>
```

鍏嬮殕鍚庣洰褰曠粨鏋勫涓嬶紝鍙洿鎺ヤ娇鐢細

```
鈹溾攢鈹€ README.md
鈹溾攢鈹€ portable/          # 渚挎惡鐗堬紝鍙屽嚮 workOrder.exe 杩愯
鈹斺攢鈹€ installer/         # 瀹夎鍖咃紙setup.exe / msi锛?```

涔熷彲鍦?GitHub 涓婂皢鍒嗘敮鍒囨崲涓?`workOrder-release` 鍚庝笅杞?ZIP銆?
---

## 涓€閿繍琛岋紙鏈€缁堢敤鎴凤級

鑻ヤ綘鎷垮埌鐨勬槸宸叉墦鍖呯殑 **`portable`** 鏂囦欢澶癸紝鏃犻渶瀹夎 Node.js 鎴?Rust锛?
1. 瑙ｅ帇鏁翠釜 `portable` 鏂囦欢澶瑰埌浠绘剰浣嶇疆锛堣矾寰勫敖閲忎笉鍚壒娈婂瓧绗︼級
2. 鍙屽嚮 **`workOrder.exe`**锛屾垨鍙屽嚮 **`鍚姩 workOrder.bat`**
3. 棣栨杩愯浼氬湪鍚岀洰褰曚笅鑷姩鍒涘缓 `data/workorder.db` 淇濆瓨鏁版嵁

### 鐩綍璇存槑

```
portable/
鈹溾攢鈹€ workOrder.exe          # 涓荤▼搴忥紝鍙屽嚮杩愯
鈹溾攢鈹€ 鍚姩 workOrder.bat     # 鍚姩鑴氭湰锛堟晥鏋滃悓涓婏級
鈹斺攢鈹€ data/                  # 鏁版嵁鐩綍锛堥娆¤繍琛屽悗鐢熸垚 workorder.db锛?```

### 绯荤粺瑕佹眰

- Windows 10 鎴栨洿楂樼増鏈紙闇€ WebView2 杩愯鏃讹紝Win10+ 閫氬父宸查瑁咃級
- 鑻ユ棤娉曞惎鍔紝鍙敼鐢ㄥ畨瑁呭寘锛堣涓嬫枃锛?
### 瀹夎鍖呮柟寮忥紙鍙€夛級

鑻ユ彁渚涚殑鏄?`installer` 鐩綍锛?
| 鏂囦欢 | 璇存槑 |
|------|------|
| `workOrder_1.1.0_x64-setup.exe` | NSIS 瀹夎绋嬪簭锛屾帹鑽愶紱浼氳嚜鍔ㄥ鐞?WebView2 |
| `workOrder_1.1.0_x64_en-US.msi` | MSI 瀹夎鍖?|

瀹夎鍚庝粠寮€濮嬭彍鍗曟垨妗岄潰蹇嵎鏂瑰紡鍚姩銆傛暟鎹粯璁や繚瀛樺湪瀹夎鐩綍鏃佺殑 `data/` 涓嬨€?
---

## 鐗堟湰鍗囩骇鎸囧崡锛堟渶缁堢敤鎴凤級

褰撳墠鐗堟湰**涓嶄細鑷姩鏇存柊**锛岄渶瑕佹墜鍔ㄤ笅杞芥柊鐗堟湰骞惰鐩栧畨瑁呫€備粠 **v1.0 鍗囩骇鍒?1.1** 鐨勮缁嗚鏄庤 **[docs/upgrade/v1.0-to-1.1.md](docs/upgrade/v1.0-to-1.1.md)**銆?
鍗囩骇鍚庨娆″惎鍔ㄦ椂锛岀▼搴忎細鑷姩杩佺Щ鏁版嵁搴撳苟鐢熸垚 `status_config.json`锛?*鏃犻渶鎵嬪姩鏀规暟鎹簱**銆?
### 浠?v1.0 蹇€熷崌绾э紙鎽樿锛?
1. 锛堟帹鑽愶級鍦?v1.0 **璁剧疆 鈫?澶囦唤** 瀵煎嚭 ZIP锛屾垨澶嶅埗 `data/` 鏂囦欢澶?2. **瀹屽叏閫€鍑?* workOrder
3. 鐢?v1.1 瀹夎鍖呰鐩栧畨瑁咃紝鎴栦粎鐢ㄦ柊 `workOrder.exe` 鏇挎崲鏃?exe锛?*淇濈暀 `data/`**锛?4. 鍚姩 v1.1 鈥?鑷姩瀹屾垚杩佺Щ

鍙€夛細渚挎惡鍖呬腑鍙屽嚮 **`杩佺Щ鏁版嵁.bat`**锛屽湪涓嶆墦寮€涓荤晫闈㈢殑鎯呭喌涓嬪厛杩佺Щ锛堜笌鍚姩鏃堕€昏緫鐩稿悓锛夈€?
### 鍗囩骇鍓?
1. **瀹屽叏閫€鍑?* workOrder锛堢‘璁や换鍔＄鐞嗗櫒涓棤 `workOrder.exe` 杩涚▼锛?2. **澶囦唤鏁版嵁**锛堟帹鑽愶級锛氬鍒?`workorder.db` 鍒板畨鍏ㄤ綅缃?
鏁版嵁鏂囦欢甯歌浣嶇疆锛?
| 浣跨敤鏂瑰紡 | 榛樿璺緞 |
|----------|----------|
| 渚挎惡鐗?| `portable/data/workorder.db` |
| 瀹夎鐗?| `{瀹夎鐩綍}/data/workorder.db` |
| 搴旂敤鍐呰缃繃鏁版嵁鐩綍 | 璁剧疆椤垫樉绀虹殑璺緞 |
| 鐜鍙橀噺 | `WORKORDER_DATA_DIR` 鎸囧悜鐨勭洰褰?|

### 瀹夎鐗堝崌绾э紙setup.exe / msi锛?
閫傜敤浜庡綋鍒濋€氳繃 `installer` 鐩綍涓殑瀹夎绋嬪簭瀹夎鐨勭敤鎴枫€?
1. 浠?`workOrder-release` 鍒嗘敮鎴?Release 椤甸潰涓嬭浇鏂扮増 `installer/`
2. 杩愯鏂扮増 **`workOrder_x.x.x_x64-setup.exe`**锛堟帹鑽愶級鎴?MSI
3. 瀹夎鍒?*鍘熷畨瑁呰矾寰?*锛堝畨瑁呯▼搴忎細璇嗗埆鏃х増骞惰鐩栧崌绾э級
4. 浠庡紑濮嬭彍鍗曟垨妗岄潰蹇嵎鏂瑰紡鍚姩

鍗囩骇鍚庝互涓嬪唴瀹逛細淇濈暀锛?
| 鍐呭 | 鏄惁淇濈暀 |
|------|----------|
| `data/workorder.db`锛堜唬鍔炰笌澶勭疆杩囩▼锛?| 淇濈暀 |
| `data/status_config.json`锛坴1.1 鏂板锛岄娆″惎鍔ㄨ嚜鍔ㄧ敓鎴愶級 | 淇濈暀鎴栨柊寤?|
| `data/attachments/`锛堝浘鐗囬檮浠讹級 | 淇濈暀 |
| `settings.json`锛堣嫢瀛樺湪锛岃褰曡嚜瀹氫箟鏁版嵁鐩綍锛?| 淇濈暀 |
| 绋嬪簭 exe | 鏇挎崲涓烘柊鐗堟湰 |

### 渚挎惡鐗堝崌绾?
閫傜敤浜庤В鍘?`portable` 鏂囦欢澶圭洿鎺ヨ繍琛岀殑鐢ㄦ埛銆?
1. 閫€鍑虹▼搴?2. 鐢ㄦ柊鐗?**`workOrder.exe`** 瑕嗙洊鏃ф枃浠?3. **淇濈暀**鍚岀洰褰曚笅鐨?`data/` 鏂囦欢澶瑰拰 `settings.json`锛堝鏈夛級
4. 鍙屽嚮 `workOrder.exe` 鎴?`鍚姩 workOrder.bat` 鍚姩

```
portable/
鈹溾攢鈹€ workOrder.exe          鈫?浠呮浛鎹㈡鏂囦欢
鈹溾攢鈹€ workOrder-migrate.exe  鈫?v1.1 鏂板锛堝彲閫夛級
鈹溾攢鈹€ 鍚姩 workOrder.bat
鈹溾攢鈹€ 杩佺Щ鏁版嵁.bat           鈫?v1.1 鏂板锛堝彲閫夛級
鈹溾攢鈹€ settings.json          鈫?淇濈暀锛堣嫢瀛樺湪锛?鈹斺攢鈹€ data/                  鈫?淇濈暀
    鈹溾攢鈹€ workorder.db
    鈹溾攢鈹€ attachments/       鈫?鑻ユ湁鍥剧墖闄勪欢
    鈹斺攢鈹€ status_config.json 鈫?v1.1 棣栨鍚姩鍚庣敓鎴?```

### 浠庡畨瑁呯増鍒囨崲鍒颁究鎼虹増锛堝彲閫夛級

1. 鍦ㄦ棫鐗堜腑鎵撳紑銆岃缃€嶏紝纭褰撳墠鏁版嵁鐩綍
2. 灏?`data/` 鏂囦欢澶癸紙鍚?`workorder.db`锛夊鍒跺埌鏂拌В鍘嬬殑 `portable/data/`
3. 鑻ュ畨瑁呯洰褰曟梺鏈?`settings.json`锛屼竴骞跺鍒跺埌 `portable/` 鐩綍
4. 鐢ㄤ究鎼虹増鍚姩骞舵牳瀵规暟鎹槸鍚﹀畬鏁?
### 鍗囩骇鍚庨獙璇?
- 鎵撳紑鑻ュ共鏃т唬鍔烇紝纭鍒楄〃涓庤鎯呮甯?- 妫€鏌ャ€屽缃繃绋嬨€嶆槸鍚﹀畬鏁达紙鏍囬銆佺姸鎬併€佸睍寮€鍚庣殑璇︾粏鍐呭锛?- 鑻ュ紓甯革紝鍏抽棴绋嬪簭鍚庣敤澶囦唤鐨?`workorder.db` 杩樺師

### 甯歌闂

**Q锛氬崌绾т細涓㈠け鏁版嵁鍚楋紵**  
A锛氭甯歌鐩栧畨瑁呬笉浼氥€傚彧瑕?`data/workorder.db` 鏈鍒犻櫎锛屾暟鎹細淇濈暀锛涢娆″惎鍔ㄤ細鑷姩鎵ц鏁版嵁搴撹縼绉汇€?
**Q锛氶渶瑕佸嵏杞芥棫鐗堝啀瑁呮柊鐗堝悧锛?*  
A锛氫笉闇€瑕併€傜洿鎺ヨ繍琛屾柊鐗堝畨瑁呯▼搴忚鐩栧嵆鍙€?
**Q锛氬彲浠ュ洖閫€鍒版棫鐗堟湰鍚楋紵**  
A锛氬彲浠ャ€備繚鐣欏崌绾у墠鐨?`workorder.db` 澶囦唤锛涜嫢鏂扮増鏈夐棶棰橈紝瑁呭洖鏃х増 exe 鍚庨€氬父浠嶅彲鎵撳紑鏁版嵁搴撱€?
**Q锛氬畨瑁呭埌 Program Files 鍚庤缃〉鏀逛笉浜嗘暟鎹洰褰曪紵**  
A锛氳鐩綍鍙兘鏃犲啓鍏ユ潈闄愩€傚崌绾у墠璇峰湪鏃х増銆岃缃€嶄腑纭瀹為檯鏁版嵁璺緞锛屾垨鏀圭敤渚挎惡鐗堝苟灏?`data/` 鏀惧湪鏈夊啓鏉冮檺鐨勪綅缃€?
---

## 浠庢簮鐮佹墦鍖咃紙寮€鍙戣€咃級

### 鐜瑕佹眰

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/)锛坄rustup` 瀹夎鍚庨噸鍚粓绔級
- Windows锛歔Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)锛堝嬀閫夈€屼娇鐢?C++ 鐨勬闈㈠紑鍙戙€嶏級

### 瀹夎渚濊禆

```bash
npm install
```

### 寮€鍙戞ā寮?
```bash
npm run tauri dev
```

寮€鍙戞椂鏁版嵁鏂囦欢浣嶄簬椤圭洰鏍圭洰褰?`data/workorder.db`銆?
淇敼 Rust Command 鎴?Model 鍚庯紝杩愯 `npm run bindings` 閲嶆柊鐢熸垚 `src/bindings.ts`锛坉ebug 妯″紡鍚姩鏃朵篃浼氳嚜鍔ㄥ鍑猴級銆傛洿澶氬懡浠よ [寮€鍙戣€呭父鐢ㄥ懡浠(docs/dev-commands.md)銆?
### 鎵撳寘涓哄彲鍒嗗彂鐗堟湰

浠婚€夊叾涓€锛?
```bash
# 鏂瑰紡涓€锛歯pm 鑴氭湰
npm run package:win

# 鏂瑰紡浜岋細鍙屽嚮椤圭洰鏍圭洰褰曠殑 鎵撳寘.bat
```

鎵撳寘瀹屾垚鍚庯紝浜х墿浣嶄簬 `release/`锛?
```
release/
鈹溾攢鈹€ portable/              # 渚挎惡鐗堬紝鍙洿鎺?zip 鍒嗗彂缁欎粬浜?鈹?  鈹溾攢鈹€ workOrder.exe
鈹?  鈹溾攢鈹€ 鍚姩 workOrder.bat
鈹?  鈹斺攢鈹€ data/
鈹斺攢鈹€ installer/             # 瀹夎鍖?    鈹溾攢鈹€ workOrder_1.1.0_x64-setup.exe
    鈹斺攢鈹€ workOrder_1.1.0_x64_en-US.msi
```

灏?**`release/portable`** 鏂囦欢澶瑰帇缂╀负 zip 鍗冲彲鍒嗕韩缁欎粬浜轰竴閿繍琛屻€?
### 鏇存柊 `workOrder-release` 鍙戝竷鍒嗘敮

`workOrder-release` 鍒嗘敮**浠呭寘鍚彲杩愯浜х墿**锛堟棤婧愮爜锛夛紝渚涗粬浜虹洿鎺ュ厠闅嗕娇鐢ㄣ€傛湰鍦版墦鍖呭畬鎴愬悗锛屽彲鎵嬪姩鏇存柊璇ュ垎鏀細

```bash
npm run package:win
git checkout workOrder-release
# 鐢?release/portable 涓?release/installer 瑕嗙洊鍒嗘敮鏍圭洰褰曞搴旀枃浠跺す
git add portable installer README.md
git commit -m "鍙戝竷 workOrder x.x.x"
git checkout main
```

### 鏁版嵁鐩綍

| 鍦烘櫙 | 鏁版嵁璺緞 |
|------|----------|
| 渚挎惡鐗堣繍琛?| `portable/data/workorder.db` |
| 瀹夎鐗堣繍琛?| `{瀹夎鐩綍}/data/workorder.db` |
| 搴旂敤鍐呰缃?| 璁剧疆椤典慨鏀瑰悗鍐欏叆 exe 鏃?`settings.json` |
| 寮€鍙戞ā寮?| 椤圭洰鏍?`data/workorder.db` |
| 鑷畾涔夎矾寰?| 璁剧疆鐜鍙橀噺 `WORKORDER_DATA_DIR` |

---

## 鏇村鏂囨。

- [寮€鍙戣€呭父鐢ㄥ懡浠(docs/dev-commands.md)
- [鍚庣鏋舵瀯](docs/backend.md)
- [Tauri Command API](docs/api/commands.md)
- [闇€姹傛枃妗(plan/闇€姹傛枃妗?md)
- [鎶€鏈€夊瀷](plan/鎶€鏈€夊瀷.md)
- [瀹炵幇璁″垝](plan/瀹炵幇璁″垝.md)
