fn main() {
  cc::Build::new()
      .file("sqlite/sqlite3.c")  // Ruta al archivo fuente
      .compile("sqlite3");       // Nombre de la biblioteca generada
}