message-cant-embed =
    Ruffle tidak dapat menjalankan Flash yang disematkan di halaman ini.
    Anda dapat mencoba membuka berkas di tab terpisah, untuk menghindari masalah ini.
message-restored-from-bfcache = Browser anda memulihkan konten Flash ini dari sesi sebelumnya. Untuk memulai ulang, muat ulang laman ini.
panic-title = Terjadi kesalahan :(
more-info = Info lebih lanjut
run-anyway = Jalankan
continue = Lanjutkan
report-bug = Laporkan Bug
update-ruffle = Perbarui Ruffle
ruffle-demo = Demo Web
ruffle-desktop = Aplikasi Desktop
ruffle-wiki = Kunjungi Wiki Ruffle
enable-hardware-acceleration = Sepertinya akselerasi perangkat keras dimatikan. Meskipun Ruffle akan bekerja, Ruffle mungkin akan bekerja sangat lambat. Anda dapat mencari tahu cara mengaktifkan akselerasi perangkat keras dengan mengikuti tautan dibawah:
enable-hardware-acceleration-link = SSD - Akselerasi Perangkat Keras Chrome
view-error-details = Tunjukkan Keterangan Error
open-in-new-tab = Buka di Tab Baru
click-to-unmute = Tekan untuk menyalakan suara
clipboard-message-title = Menyalin dan Menempel di Ruffle
clipboard-message-description =
    { $variant ->
       *[unsupported] Browser Anda tidak mendukung akses penuh papan kerani,
        [access-denied]  Akses untuk papan kerani tidak diizinkan,
    } Anda tetap dapat menggunakan pintasan ini:
clipboard-message-copy = { " " } untuk menyalin
clipboard-message-cut = { " " } untuk memotong
clipboard-message-paste = { " " } untuk menempel
error-canvas-reload = Tidak dapat memuat dengan renderer canvas saat renderer canvas sedang digunakan.
error-file-protocol =
    Sepertinya anda menjalankan Ruffle di protokol "file:".
    Ini tidak berfungsi karena browser memblokir fitur ini dengan alasan keamanan.
    Sebagai gantinya, kami mengajak anda untuk membuat server lokal, menggunakan demo web atau aplikasi desktop.
error-javascript-config =
    Ruffle mengalami masalah besar karena konfigurasi JavaScript yang salah.
    Jika Anda adalah administrator server ini, kami mengajak Anda untuk memeriksa detail kesalahan untuk mengetahui parameter mana yang salah.
    Anda juga dapat membaca wiki Ruffle untuk mendapatkan bantuan.
error-wasm-not-found =
    Ruffle gagal memuat komponen berkas ".wasm" yang diperlukan.
    Jika Anda adalah administrator server ini, pastikan berkas telah diunggah dengan benar.
    Jika masalah terus berlanjut, Anda mungkin perlu menggunakan pengaturan "publicPath": silakan baca wiki Ruffle untuk mendapatkan bantuan.
error-wasm-mime-type =
    Ruffle mengalami masalah ketika mencoba melakukan inisialisasi.
    Server web ini tidak melayani berkas ".wasm" dengan tipe MIME yang benar.
    Jika Anda adalah administrator server ini, silakan baca wiki Ruffle untuk mendapatkan bantuan.
error-invalid-swf =
    Ruffle tidak dapat membaca berkas yang diminta.
    Kemungkinan terbesar berkas yang diminta bukan berkas SWF valid.
error-swf-fetch =
    Ruffle gagal memuat berkas SWF Flash.
    Kemungkinan berkas tersebut sudah tidak ada, sehingga tidak dapat dimuat oleh Ruffle.
    Coba hubungi administrator situs web ini untuk mendapatkan bantuan.
error-swf-cors =
    Ruffle gagal memuat berkas SWF Flash.
    Akses untuk memuat kemungkinan telah diblokir oleh kebijakan CORS.
    Jika Anda adalah administrator server ini, silakan baca wiki Ruffle untuk mendapatkan bantuan.
error-wasm-cors =
    Ruffle gagal memuat komponen berkas ".wasm" yang diperlukan.
    Akses untuk mengambil kemungkinan telah diblokir oleh kebijakan CORS.
    Jika Anda adalah administrator server ini, silakan baca wiki Ruffle untuk mendapatkan bantuan.
error-wasm-invalid =
    Ruffle mengalami masalah besar ketika mencoba melakukan inisialisasi.
    Sepertinya halaman ini memiliki berkas yang hilang atau tidak valid untuk menjalankan Ruffle.
    Jika Anda adalah administrator server ini, silakan baca wiki Ruffle untuk mendapatkan bantuan.
error-wasm-download =
    Ruffle mengalami masalah besar ketika mencoba melakukan inisialisasi.
    Hal ini sering kali dapat teratasi dengan sendirinya, sehingga Anda dapat mencoba memuat ulang halaman.
    Jika tidak, silakan hubungi administrator situs web ini.
error-wasm-disabled-on-edge =
    Ruffle gagal memuat komponen berkas ".wasm" yang diperlukan.
    Untuk mengatasinya, coba buka pengaturan peramban Anda, klik "Privasi, pencarian, dan layanan", turun ke bawah, dan matikan "Tingkatkan keamanan Anda di web".
    Ini akan memungkinkan browser Anda memuat berkas ".wasm" yang diperlukan.
    Jika masalah berlanjut, Anda mungkin harus menggunakan browser yang berbeda.
error-wasm-unsupported-browser =
    Browser yang anda gunakan tidak mendukung ekstensi WebAssembly yang diperlukan Ruffle untuk berjalan.
    Silakan menggunakan browser yang mendukung.
    Anda dapat menemukan daftar browser yang didukung di Wiki.
error-javascript-conflict =
    Ruffle mengalami masalah besar ketika mencoba melakukan inisialisasi.
    Sepertinya situs web ini menggunakan kode JavaScript yang bertentangan dengan Ruffle.
    Jika Anda adalah administrator server ini, kami mengajak Anda untuk mencoba memuat berkas pada halaman kosong.
error-javascript-conflict-outdated = Anda juga dapat mencoba mengunggah versi Ruffle yang lebih baru yang mungkin dapat mengatasi masalah ini (versi saat ini sudah kedaluwarsa: { $buildDate }).
error-csp-conflict =
    Ruffle mengalami masalah besar ketika mencoba melakukan inisialisasi.
    Kebijakan Keamanan Konten server web ini tidak mengizinkan komponen ".wasm" yang diperlukan untuk dijalankan.
    Jika Anda adalah administrator server ini, silakan baca wiki Ruffle untuk mendapatkan bantuan.
error-url-invalid =
    Ruffle tidak dapat memuat berkas Flash SWF.
    Kemungkinan terbesar adalah tautan yang diberikan kepada Ruffle untuk memuat berkas SWF tidak valid.
error-unknown =
    Ruffle mengalami masalah besar saat menampilkan konten Flash ini.
    { $outdated ->
        [true] Jika Anda administrator server ini, cobalah untuk mengganti versi Ruffle yang lebih baru (versi saat ini sudah kedaluwarsa: { $buildDate }).
       *[false] Hal ini seharusnya tidak terjadi, jadi kami sangat menghargai jika Anda dapat melaporkan bug ini!
    }
