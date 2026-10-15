# Kusuri Web Dashboard — Requirements Specification

## 1. Overview

Kusuri Web Dashboard adalah antarmuka web responsif untuk mengelola akun, perangkat yang terhubung, sinkronisasi data, transfer file, dan keamanan pengguna Kusuri.

Dashboard berfungsi sebagai control plane untuk pengguna, sedangkan client Android, Windows, dan Linux tetap menjadi aplikasi utama untuk menjalankan fitur yang membutuhkan integrasi dengan sistem operasi.

Dashboard juga menyediakan area administrasi khusus untuk operator instance Kusuri self-hosted. Area ini menggunakan sistem otorisasi terpisah dari akun pengguna biasa.

## 2. Objectives

- Mempermudah pengguna mengelola seluruh perangkat melalui satu antarmuka.
- Menyediakan pemantauan status perangkat dan aktivitas sinkronisasi secara real-time.
- Mempermudah pairing dan pengelolaan perangkat baru.
- Menyediakan akses terhadap riwayat clipboard dan file yang dibagikan.
- Memberikan kontrol atas keamanan akun, sesi, dan akses perangkat.
- Menyediakan antarmuka administrasi untuk instance self-hosted.
- Menjaga konsistensi pengalaman pengguna di desktop, tablet, dan perangkat mobile.
- Menghindari duplikasi implementasi fitur yang lebih sesuai dijalankan melalui client native.

## 3. Scope

### 3.1. Included

- User Dashboard.
- Device Management.
- Device Pairing.
- Device Detail dan Telemetry.
- Clipboard History.
- File Management.
- Command History.
- Account Settings.
- Security Settings.
- Instance Admin Panel.
- Real-time updates melalui WebSocket.
- Responsive layout.
- Loading, empty, error, dan offline states.
- Authentication dan authorization pada sisi frontend.
- Integrasi dengan API yang tersedia.

### 3.2. Excluded

- Implementasi ulang backend.
- Implementasi client Android, Windows, atau Linux.
- Penggantian fungsi native client dengan browser API.
- Eksekusi remote command langsung dari browser tanpa melalui mekanisme kontrol yang tersedia.
- Database management langsung melalui browser.
- Pengelolaan infrastruktur host secara arbitrer.
- Sistem billing dan subscription.
- Multi-tenant SaaS management untuk organisasi eksternal.
- Implementasi ulang protokol sinkronisasi.

## 4. User Roles

### 4.1. User

Pengguna biasa yang memiliki akun Kusuri.

Permissions:
- Mengakses dashboard milik sendiri.
- Melihat dan mengelola perangkat milik sendiri.
- Membuat pairing code.
- Melihat telemetri perangkat yang diizinkan.
- Melihat dan menghapus riwayat clipboard milik sendiri.
- Melihat, mengunduh, dan menghapus file sesuai hak akses.
- Melihat riwayat remote command milik sendiri.
- Mengelola profil, sesi, dan keamanan akun.

### 4.2. Instance Administrator

Operator yang bertanggung jawab mengelola instance Kusuri self-hosted.

Permissions:
- Mengakses halaman administrasi instance.
- Melihat kesehatan layanan dan penggunaan resource yang tersedia.
- Melihat status koneksi dan metrik operasional.
- Melihat log aplikasi yang diizinkan.
- Mengelola konfigurasi operasional yang secara eksplisit didukung.
- Mengelola pengguna apabila instance mengaktifkan fitur tersebut.
- Melihat status sinkronisasi katalog perangkat.
- Mengakses informasi diagnostik tanpa memperoleh akses otomatis ke konten pribadi pengguna.

### 4.3. Authorization Rules

- Role administrator tidak boleh diberikan hanya karena pengguna berhasil login.
- Semua akses sensitif wajib divalidasi oleh server.
- Pengguna tidak boleh mengakses perangkat milik akun lain.
- Administrator instance tidak otomatis memiliki hak untuk membaca clipboard atau mengunduh file pribadi pengguna.
- UI harus menyembunyikan tindakan yang tidak diizinkan, tetapi penyembunyian UI tidak menggantikan validasi server.

## 5. Application Layout

### 5.1. Desktop Layout

- Sidebar navigasi.
- Header dengan judul halaman dan menu akun.
- Area konten utama.
- Notifikasi aktivitas dan status koneksi.
- Breadcrumb untuk halaman bertingkat jika diperlukan.

### 5.2. Mobile Layout

- Navigasi berbentuk drawer atau bottom navigation.
- Sidebar tidak boleh menghabiskan ruang konten.
- Tabel harus dapat beradaptasi menjadi card list.
- Dialog dan formulir harus nyaman digunakan dengan sentuhan.
- Informasi penting tidak boleh bergantung pada hover.
- Tindakan destruktif harus tetap mudah dikenali dan dikonfirmasi.

### 5.3. Navigation Structure

- Overview
- Devices
- Pair Device
- Clipboard
- Files
- Command History
- Settings
  - Account
  - Security
  - Privacy
- Instance Administration
  - System Overview
  - Services
  - Logs
  - Users
  - Configuration
  - Device Catalog

Menu Instance Administration hanya tersedia bagi administrator yang memiliki izin sesuai.

## 6. Functional Requirements

### 6.1. Authentication

Fitur:
- Login.
- Logout.
- Session persistence sesuai kebijakan keamanan.
- Penanganan token kedaluwarsa.
- Redirect ke halaman login ketika sesi tidak lagi valid.
- Tampilan error autentikasi yang informatif.
- Proteksi halaman yang membutuhkan autentikasi.
- Penanganan kegagalan koneksi dan server.

Acceptance Criteria:
- Pengguna yang belum login tidak dapat mengakses halaman privat.
- Pengguna yang logout tidak dapat melanjutkan akses menggunakan sesi yang telah dicabut.
- Token tidak ditampilkan pada URL atau log browser.
- Kegagalan autentikasi tidak menampilkan informasi internal server.

### 6.2. Overview Dashboard

Menampilkan:
- Jumlah perangkat terdaftar.
- Jumlah perangkat online dan offline.
- Aktivitas perangkat terbaru.
- Status sinkronisasi terakhir.
- Ringkasan clipboard terbaru.
- Ringkasan file terbaru.
- Ringkasan remote command terbaru.
- Status koneksi real-time.

Acceptance Criteria:
- Informasi hanya mencakup data yang dapat diakses oleh pengguna.
- Status koneksi diperbarui secara real-time jika koneksi tersedia.
- Ketika koneksi real-time terputus, dashboard menampilkan indikator koneksi terputus.
- Data yang belum tersedia ditampilkan sebagai unavailable, bukan sebagai nilai nol yang menyesatkan.
- Pengguna dapat membuka halaman detail dari ringkasan yang relevan.

### 6.3. Device Management

Menampilkan daftar perangkat dengan informasi:
- Nama perangkat.
- Tipe perangkat.
- Nama model pemasaran jika tersedia.
- Status online/offline.
- Sistem operasi.
- Waktu aktivitas terakhir jika tersedia.
- Status baterai jika didukung.
- Status sinkronisasi.

Actions:
- Melihat detail perangkat.
- Mengubah nama perangkat.
- Melihat spesifikasi perangkat.
- Melihat telemetri perangkat.
- Melihat riwayat aktivitas yang tersedia.
- Menghapus atau memutuskan perangkat dari akun.
- Mencabut token perangkat.
- Menerbitkan token baru melalui mekanisme yang didukung.

Acceptance Criteria:
- Pengguna hanya melihat perangkat miliknya.
- Perangkat offline tetap dapat ditampilkan dengan status yang jelas.
- Perubahan nama memiliki validasi input.
- Penghapusan perangkat memerlukan konfirmasi.
- Pencabutan token menampilkan hasil operasi secara jelas.
- Dashboard tidak mengasumsikan bahwa perangkat online hanya karena pernah terdaftar.

### 6.4. Device Detail & Telemetry

Halaman detail perangkat menyediakan:
- Identitas dan tipe perangkat.
- Model pemasaran dan spesifikasi hardware jika tersedia.
- Chipset, CPU, GPU, display, kamera, dan baterai jika tersedia.
- Informasi sistem operasi.
- RAM total dan penggunaan RAM.
- Storage total dan penggunaan storage.
- Persentase dan suhu baterai jika tersedia.
- Status charging.
- IP address jika tersedia dan diizinkan.
- Waktu telemetri terakhir.
- Riwayat telemetri jika didukung.
- Riwayat command terkait perangkat jika diizinkan.

Acceptance Criteria:
- Nilai telemetri menampilkan satuan yang sesuai.
- Data lama diberi penanda waktu.
- Data yang tidak tersedia tidak ditampilkan sebagai nilai faktual.
- Identitas sensitif seperti IMEI disembunyikan secara default dan hanya ditampilkan jika benar-benar diperlukan serta diizinkan.
- Telemetri tidak dianggap real-time apabila tidak ada pembaruan baru.

### 6.5. Device Pairing

Fitur:
- Membuat pairing code.
- Menampilkan kode pairing.
- Menampilkan waktu kedaluwarsa.
- Menampilkan sisa masa berlaku.
- Menampilkan petunjuk memasukkan kode pada client.
- Menampilkan status pairing.
- Menampilkan hasil pairing dan identitas perangkat yang berhasil terhubung.

Acceptance Criteria:
- Pairing code mengikuti masa berlaku dan batas percobaan yang diberlakukan server.
- Kode kedaluwarsa tidak dapat digunakan kembali.
- Kode hanya ditampilkan selama diperlukan.
- Kode tidak dicatat ke analytics atau log frontend.
- Pengguna mendapat informasi ketika pairing berhasil, gagal, atau kedaluwarsa.
- Penerbitan pairing code baru tidak boleh diam-diam memperpanjang atau mengganti sesi yang masih aktif tanpa penjelasan.

### 6.6. Clipboard History

Fitur:
- Menampilkan riwayat clipboard.
- Menampilkan konten teks.
- Menampilkan tipe konten.
- Menampilkan sumber perangkat.
- Menampilkan waktu penerimaan.
- Menampilkan pagination atau infinite scrolling.
- Menyalin konten melalui tindakan eksplisit pengguna.
- Menghapus entri clipboard.
- Menampilkan pembaruan clipboard baru secara real-time.

Acceptance Criteria:
- Riwayat hanya mencakup data yang dapat diakses pengguna.
- Konten panjang dapat dibaca tanpa merusak layout.
- Konten tidak dieksekusi sebagai HTML.
- Clipboard baru tidak langsung menimpa konten yang sedang dibaca tanpa notifikasi yang jelas.
- Penghapusan memerlukan konfirmasi jika bersifat permanen.
- Tidak ada konten clipboard yang dikirim ke layanan analitik pihak ketiga.

### 6.7. File Management

Fitur:
- Menampilkan daftar file yang dibagikan.
- Menampilkan nama file.
- Menampilkan ukuran file.
- Menampilkan waktu upload.
- Menampilkan sumber dan tujuan perangkat jika tersedia.
- Mengunduh file.
- Menghapus file.
- Menampilkan status upload atau ketersediaan file jika disediakan oleh API.
- Menampilkan error ketika file tidak lagi tersedia.

Acceptance Criteria:
- Link unduhan hanya dapat digunakan oleh pengguna yang berwenang.
- File besar tidak perlu dimuat seluruhnya ke memori browser.
- Nama file diperlakukan sebagai data tidak tepercaya.
- File tidak otomatis dibuka atau dieksekusi.
- Penghapusan memerlukan konfirmasi.
- Status upload tidak boleh ditampilkan sebagai berhasil sebelum server mengonfirmasi keberhasilan.

### 6.8. Command History

Fitur:
- Menampilkan daftar remote command.
- Menampilkan perangkat target.
- Menampilkan jenis command.
- Menampilkan waktu pengiriman.
- Menampilkan status command.
- Menampilkan hasil eksekusi jika tersedia.
- Melihat detail response command.
- Memfilter berdasarkan perangkat, jenis command, dan status.

Acceptance Criteria:
- Command history hanya mencakup data yang berhak dilihat pengguna.
- Status pending, completed, failed, dan status lain harus mengikuti kontrak API.
- Dashboard membedakan command terkirim dari command yang benar-benar berhasil dieksekusi.
- Response yang mengandung data sensitif tidak ditampilkan tanpa pembatasan.
- Command tidak boleh dieksekusi ulang hanya karena pengguna membuka halaman detail.

### 6.9. Account Settings

Fitur:
- Melihat informasi akun.
- Mengubah informasi profil jika didukung.
- Mengubah password jika endpoint tersedia.
- Melihat sesi aktif jika didukung.
- Logout dari sesi saat ini.
- Mencabut sesi lain jika didukung.

Acceptance Criteria:
- Perubahan profil memiliki validasi.
- Formulir menampilkan loading dan error state.
- Perubahan password tidak mengekspos password sebelumnya.
- Operasi keamanan membutuhkan autentikasi dan otorisasi yang sesuai.

### 6.10. Security & Privacy Settings

Fitur:
- Mengelola sesi login.
- Mengelola token perangkat.
- Menampilkan informasi perangkat yang memiliki akses.
- Mengatur preferensi privasi jika tersedia.
- Mengatur retensi clipboard dan file jika didukung.
- Mengatur izin remote action jika tersedia.
- Menampilkan informasi aktivitas keamanan jika didukung.

Acceptance Criteria:
- Token rahasia tidak pernah ditampilkan kembali setelah proses penerbitan selesai.
- Pengguna mendapat konfirmasi setelah token atau sesi berhasil dicabut.
- Pengaturan yang belum didukung server tidak ditampilkan sebagai pengaturan aktif.
- Perubahan kebijakan akses berlaku setelah server mengonfirmasi keberhasilan.

## 7. Instance Admin Panel

Admin Panel merupakan bagian dari aplikasi web yang sama, tetapi terpisah dari navigasi dan otorisasi pengguna biasa.

### 7.1. System Overview

Menampilkan:
- Status kesehatan instance.
- Status backend dan database.
- Jumlah koneksi WebSocket aktif jika tersedia.
- Jumlah pengguna dan perangkat terdaftar jika tersedia.
- Penggunaan CPU dan RAM jika tersedia.
- Kapasitas dan penggunaan storage jika tersedia.
- Status proses sinkronisasi katalog perangkat.
- Waktu uptime jika tersedia.

### 7.2. Services

Fitur:
- Melihat status layanan.
- Melihat status koneksi database.
- Melihat status integrasi eksternal.
- Melihat error operasional yang tersedia.
- Menampilkan waktu pemeriksaan terakhir.

Dashboard tidak boleh menyediakan terminal arbitrer atau eksekusi shell pada host.

### 7.3. Logs

Fitur:
- Melihat log aplikasi yang diizinkan.
- Memfilter berdasarkan level dan rentang waktu jika didukung.
- Mencari pesan error.
- Menampilkan waktu kejadian dan sumber log.

Security Requirements:
- Password, JWT, device token, pairing code, dan data autentikasi tidak boleh ditampilkan.
- Konten clipboard dan file pribadi tidak boleh masuk ke log administratif.
- Log harus diperlakukan sebagai data tidak tepercaya.
- Log yang sensitif harus disamarkan sebelum dikirim ke frontend.

### 7.4. User Administration

Fitur:
- Melihat daftar pengguna jika diaktifkan.
- Melihat status akun.
- Melihat jumlah perangkat terkait.
- Menonaktifkan atau mengaktifkan akun jika didukung.
- Melihat metadata operasional yang diperlukan.

Batasan:
- Tidak boleh membaca password atau token mentah.
- Tidak boleh mengambil alih akun pengguna tanpa prosedur otorisasi eksplisit.
- Tidak boleh membaca konten clipboard atau file pengguna secara default.

### 7.5. Configuration

Fitur:
- Menampilkan konfigurasi operasional yang didukung.
- Mengubah batas upload jika endpoint administrasi tersedia.
- Mengatur kebijakan retensi jika didukung.
- Mengatur konfigurasi katalog perangkat jika didukung.
- Menampilkan validasi sebelum perubahan diterapkan.

Batasan:
- Secret tidak boleh ditampilkan dalam bentuk plaintext.
- Konfigurasi sensitif tidak dapat diubah melalui manipulasi frontend.
- Perubahan konfigurasi harus divalidasi dan dikonfirmasi oleh server.
- Jika konfigurasi hanya dapat diubah melalui environment variable, dashboard cukup menampilkan petunjuk operasional tanpa menyediakan editor konfigurasi palsu.

### 7.6. Device Catalog

Fitur:
- Melihat status sinkronisasi katalog.
- Melihat waktu sinkronisasi terakhir.
- Melihat status cache spesifikasi.
- Memulai sinkronisasi manual jika diizinkan.
- Melihat hasil atau error sinkronisasi.

Acceptance Criteria:
- Dashboard tidak melakukan scraping langsung dari browser.
- Sinkronisasi manual memerlukan izin administrator.
- Operasi yang sedang berlangsung menampilkan status yang jelas.
- Kegagalan sinkronisasi tidak dianggap sebagai keberhasilan.

## 8. Real-Time Requirements

- Menggunakan WebSocket untuk event yang memang tersedia.
- Menampilkan status koneksi WebSocket.
- Melakukan reconnect dengan exponential backoff dan jitter.
- Menghindari koneksi ulang tanpa batas dalam interval sangat pendek.
- Melakukan sinkronisasi ulang data penting setelah koneksi pulih.
- Mencegah event duplikat menyebabkan data ganda.
- Membersihkan koneksi saat halaman atau komponen tidak lagi aktif.
- Menghindari polling agresif jika data dapat diperoleh melalui WebSocket.
- Menyediakan fallback REST API ketika fitur real-time tidak tersedia.
- Tidak menganggap event WebSocket sebagai pengganti validasi otorisasi.

## 9. Security Requirements

- Semua koneksi produksi menggunakan HTTPS dan WSS.
- JWT disimpan menggunakan strategi yang sesuai dengan model ancaman; utamakan cookie HttpOnly, Secure, dan SameSite apabila arsitektur autentikasi mendukungnya.
- Hindari menyimpan token jangka panjang di localStorage.
- Jangan memasukkan token autentikasi ke URL.
- Gunakan tiket WebSocket berumur pendek atau mekanisme autentikasi koneksi yang aman.
- Validasi seluruh input pada frontend dan server.
- Terapkan output encoding untuk mencegah XSS.
- Cegah CSRF apabila autentikasi menggunakan cookie.
- Jangan mencatat password, token, pairing code, IMEI, atau konten privat ke console maupun analytics.
- Tampilkan konfirmasi pada tindakan destruktif.
- Terapkan least privilege pada seluruh fitur administratif.
- Jangan mempercayai role atau permission yang hanya berasal dari state frontend.
- Hindari penggunaan third-party analytics pada halaman yang menampilkan data pribadi.
- Sediakan penanganan sesi kedaluwarsa dan token yang dicabut.
- Terapkan pembatasan akses file melalui server.
- Gunakan Content Security Policy dan security headers melalui deployment.
- Hindari menampilkan stack trace, query database, atau secret internal pada pesan error.

## 10. UI/UX Requirements

- Desain modern, sederhana, dan konsisten.
- Mengutamakan keterbacaan data dan efisiensi navigasi.
- Mendukung tema terang dan gelap.
- Menggunakan status badge yang konsisten.
- Menyediakan skeleton loading untuk konten yang sedang dimuat.
- Menyediakan empty state yang menjelaskan tindakan selanjutnya.
- Menyediakan error state dengan opsi retry yang sesuai.
- Menyediakan toast atau notifikasi untuk hasil operasi.
- Memiliki dialog konfirmasi untuk tindakan destruktif.
- Menghindari animasi yang tidak diperlukan.
- Memiliki tampilan tabel dan card list yang adaptif.
- Mendukung keyboard navigation.
- Memiliki focus indicator yang terlihat.
- Memenuhi target aksesibilitas WCAG 2.2 AA sejauh memungkinkan.
- Menggunakan format tanggal, waktu, ukuran file, dan satuan hardware secara konsisten.
- Menampilkan waktu lokal pengguna dengan tetap menyediakan timestamp yang tidak ambigu jika diperlukan.

## 11. Technical Requirements

### 11.1. Frontend Stack

- React.
- TypeScript.
- Vite.
- Tailwind CSS.
- TanStack Query untuk server state.
- WebSocket API native.
- React Router atau routing library setara.
- Library form dan validasi schema bila diperlukan.
- Komponen UI reusable.
- Unit testing dan integration testing.

### 11.2. Architecture

- Menggunakan satu frontend untuk User Dashboard dan Instance Admin Panel.
- Memisahkan route, layout, dan permission untuk area user dan administrator.
- Memisahkan API client dari komponen UI.
- Menggunakan typed request dan response.
- Menggunakan validasi runtime pada data eksternal jika diperlukan.
- Menggunakan centralized error handling.
- Menggunakan centralized authentication/session handling.
- Memisahkan server state, UI state, dan connection state.
- Tidak menduplikasi business logic yang seharusnya menjadi tanggung jawab server.
- Mendukung konfigurasi base URL API untuk deployment self-hosted.
- Tidak mengharuskan layanan backend tambahan hanya untuk menyajikan frontend statis.

### 11.3. Deployment

- Frontend dapat dibangun menjadi static assets.
- Dapat di-host di belakang reverse proxy.
- Mendukung deployment pada domain atau subdomain yang sama dengan backend jika memungkinkan.
- Mendukung konfigurasi API URL untuk deployment terpisah.
- Memiliki dokumentasi build dan deployment.
- Memiliki konfigurasi environment yang jelas.
- Tidak menaruh secret backend dalam environment variable yang dikirim ke browser.
- Mendukung routing SPA melalui fallback yang sesuai.
- Mendokumentasikan kebutuhan HTTPS dan WebSocket proxy.

## 12. Performance Requirements

- Halaman awal memuat komponen penting terlebih dahulu.
- Menggunakan pagination untuk daftar yang besar.
- Menghindari pengambilan seluruh riwayat clipboard dan file sekaligus.
- Menghindari render ulang berlebihan akibat event WebSocket.
- Menggunakan lazy loading untuk halaman administratif yang jarang digunakan.
- Membatasi ukuran bundle melalui code splitting.
- Tidak mengunduh file hanya untuk menampilkan metadata.
- Tidak menyimpan seluruh telemetri historis di memori browser.
- Memiliki batas ukuran tampilan konten clipboard yang panjang.
- Mempertahankan responsivitas pada perangkat dengan spesifikasi rendah.

Target awal:
- Largest Contentful Paint (LCP) <= 2,5 detik pada kondisi pengujian yang ditetapkan.
- Interaction to Next Paint (INP) <= 200 ms pada kondisi pengujian yang ditetapkan.
- Cumulative Layout Shift (CLS) <= 0,1.
- Target tersebut dievaluasi pada persentil ke-75 untuk perangkat dan kondisi pengujian yang representatif.

## 13. Testing Requirements

### 13.1. Unit Tests
- Validasi form.
- Format data telemetri.
- Pemrosesan status koneksi.
- Parsing event WebSocket.
- Permission-based rendering.
- Error mapping.
- Utility functions.

### 13.2. Integration Tests
- Login dan logout.
- Pengambilan data dashboard.
- Device management.
- Pairing flow.
- Clipboard history.
- File listing dan download.
- Command history.
- Session expiration.
- WebSocket reconnect.
- Penanganan API error.

### 13.3. End-to-End Tests
- Pengguna login dan melihat dashboard.
- Pengguna membuat pairing code.
- Pengguna melihat perangkat setelah pairing berhasil.
- Pengguna membuka detail telemetri.
- Pengguna menyalin atau menghapus entri clipboard.
- Pengguna mengunduh atau menghapus file.
- Pengguna mencabut akses perangkat.
- Pengguna logout.
- Administrator mengakses panel administratif.
- Pengguna biasa ditolak saat membuka halaman administratif.
- Kegagalan jaringan ditampilkan dengan benar.

## 14. Delivery Phases

### Phase 1 — MVP User Dashboard

- Authentication.
- Overview.
- Device Management.
- Device Detail.
- Device Pairing.
- Clipboard History.
- File Management.
- Basic Command History.
- Responsive layout.
- WebSocket status dan reconnect.
- Error, loading, dan empty states.

### Phase 2 — Account & Security

- Account Settings.
- Session Management jika didukung.
- Token revocation.
- Security Settings.
- Privacy Settings yang benar-benar didukung.
- Perbaikan aksesibilitas dan UX.
- Pengujian end-to-end yang lebih lengkap.

### Phase 3 — Instance Admin Panel

- System Overview.
- Service Health.
- Resource Metrics jika tersedia.
- Logs.
- User Administration jika diaktifkan.
- Configuration yang didukung.
- Device Catalog Status dan Manual Sync.

### Phase 4 — Hardening & Optimization

- Audit akses dan permission.
- Security testing.
- Performance testing.
- Bundle optimization.
- Deployment documentation.
- Browser compatibility testing.
- Backup dan recovery documentation untuk operator.
- Monitoring dan diagnostik produksi.

## 15. Definition of Done

Dashboard dianggap siap untuk rilis awal apabila:

- Semua halaman Phase 1 dapat digunakan melalui API yang tersedia.
- Tidak ada data pengguna yang dapat diakses lintas akun tanpa izin.
- Seluruh operasi penting memiliki loading, success, dan error state.
- Pairing dan pengelolaan perangkat bekerja sesuai kontrak API.
- Koneksi WebSocket dapat pulih setelah gangguan jaringan.
- Token dan data sensitif tidak bocor melalui URL, console, atau log frontend.
- Tampilan responsif pada desktop dan mobile.
- Formulir memiliki validasi dan pesan error yang jelas.
- Tindakan destruktif memerlukan konfirmasi.
- Unit test, integration test, dan skenario E2E utama lulus.
- Dokumentasi instalasi, build, konfigurasi, dan deployment tersedia.
- Fitur yang belum didukung API tidak ditampilkan sebagai fitur aktif.
- User Dashboard dan Instance Admin Panel memiliki batas akses yang jelas.

## 16. Implementation Priority

Urutan implementasi yang direkomendasikan:

1. Frontend foundation, routing, layout, dan API client.
2. Authentication dan session handling.
3. Overview Dashboard.
4. Device Management dan Device Pairing.
5. Device Detail dan Telemetry.
6. Clipboard History.
7. File Management.
8. Command History.
9. Account Settings dan Security.
10. WebSocket reliability.
11. Instance Admin Panel.
12. Security hardening, testing, dan deployment documentation.

## 17. Design Principle

Kusuri Web Dashboard harus menjadi control plane yang ringan, responsif, aman, dan mudah dioperasikan.

Dashboard tidak menggantikan client native. Dashboard menyediakan visibilitas, konfigurasi, dan pengelolaan; client native menjalankan integrasi perangkat; sedangkan seluruh otorisasi dan operasi sensitif harus tetap dikendalikan oleh server.

Implementasi harus mengikuti kontrak API yang benar-benar tersedia dan tidak mengasumsikan endpoint, permission, metrik, atau konfigurasi yang belum diimplementasikan.