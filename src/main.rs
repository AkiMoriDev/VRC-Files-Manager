use eframe::egui;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

mod models;
use models::IndexedFile;

mod sqlite_store;
use sqlite_store::SqliteStore as Store;

mod crud {
    pub mod sqlite_tags;
    pub mod sqlite_categories;
}

use crate::crud::sqlite_tags::TagStore;
use crate::crud::sqlite_categories::CategoryStore;

#[derive(Serialize, Deserialize, Default, Clone)]
struct AppConfig {
    root_path: String,
}

struct MyApp {
    store: Store,
    search_query: String,
    results: Vec<IndexedFile>,
    root_path: String,

    show_tag_manager: bool,
    new_tag: String,
    edit_tag: Option<(String, String)>,
    tags: Vec<String>,
    tag_store: TagStore,

    show_category_manager: bool,
    new_category: String,
    edit_category: Option<(String, String)>,
    categories: Vec<String>,
    categories_store: CategoryStore,

		show_item_manager: bool,
    new_item_name: String,
    selected_category: Option<String>,
    selected_tags: Vec<String>,
    item_file_path: Option<String>,
    item_image_path: Option<String>,
}

const CONFIG_FILE: &str = "config.json";

impl MyApp {
    fn new(mut store: Store, tag_store: TagStore, categories_store: CategoryStore) -> Self {
        let config = Self::load_or_create_config();

        // Insertar archivo de ejemplo
        let example_path = format!("{}/ejemplo.txt", config.root_path);
        let _ = store.insert_file(&IndexedFile {
            path: example_path.clone(),
            name: "ejemplo.txt".into(),
            tags: vec!["demo".into()],
        });

        let tags = tag_store.get_tags().unwrap_or_default();
        let categories = categories_store.get_categories().unwrap_or_default();

        let mut app = Self {
            store,
            tag_store,
            categories_store,
            search_query: String::new(),
            results: Vec::new(),
            root_path: config.root_path.clone(),

						// TAGS
            show_tag_manager: false,
            new_tag: String::new(),
            edit_tag: None,
            tags,

						// CATEGORIAS
            show_category_manager: false,
            new_category: String::new(),
            edit_category: None,
            categories,
						item_file_path: None,
						item_image_path: None,
						new_item_name: String::new(),
						selected_category: None,
						selected_tags: Vec::new(),
						show_item_manager: false
        };

        // Sincronizar categorías con la carpeta principal
        app.sync_categories_with_fs();

        app
    }

    fn load_or_create_config() -> AppConfig {
        if Path::new(CONFIG_FILE).exists() {
            if let Ok(contents) = fs::read_to_string(CONFIG_FILE) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&contents) {
                    if !config.root_path.is_empty() {
                        return config;
                    }
                }
            }
        }

        let path = FileDialog::new()
            .set_title("Selecciona la carpeta principal")
            .pick_folder()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf()).display().to_string());

        let config = AppConfig { root_path: path };
        let _ = fs::write(CONFIG_FILE, serde_json::to_string_pretty(&config).unwrap());
        config
    }

    fn save_config(&self) {
        let config = AppConfig {
            root_path: self.root_path.clone(),
        };
        let _ = fs::write(CONFIG_FILE, serde_json::to_string_pretty(&config).unwrap());
    }

    /// Sincroniza categorías en DB con carpetas físicas
    fn sync_categories_with_fs(&mut self) {
        if !Path::new(&self.root_path).exists() {
            return;
        }

        let mut existing_folders = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.root_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        existing_folders.push(name.to_string());
                    }
                }
            }
        }

        let mut db_categories = self.categories_store.get_categories().unwrap_or_default();

        // Insertar carpetas nuevas en DB
        for folder in &existing_folders {
            if !db_categories.contains(folder) {
                let _ = self.categories_store.insert_category(folder);
                db_categories.push(folder.clone());
            }
        }

        // Eliminar de DB categorías que ya no existan como carpetas
        for cat in db_categories.clone() {
            if !existing_folders.contains(&cat) {
                let _ = self.categories_store.delete_category(&cat);
                db_categories.retain(|x| x != &cat);
            }
        }

        self.categories = db_categories;
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut path_changed = false;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Manage Tags").clicked() {
                    self.show_tag_manager = true;
                }
                if ui.button("Manage Categories").clicked() {
                    self.show_category_manager = true;
                }
								if ui.button("Manage Items").clicked() {
										self.show_item_manager = true;
								}
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Administrador de Archivos VRC");

            // Carpeta principal
            ui.horizontal(|ui| {
                ui.label("Carpeta principal:");
                if ui.text_edit_singleline(&mut self.root_path).changed() {
                    path_changed = true;
                }
                if ui.button("📁").clicked() {
                    if let Some(new_path) = FileDialog::new().pick_folder() {
                        self.root_path = new_path.display().to_string();
                        path_changed = true;
                    }
                }
            });

            ui.separator();

            // Buscador
            ui.horizontal(|ui| {
                ui.label("Buscar:");
                ui.text_edit_singleline(&mut self.search_query);
                if ui.button("🔍").clicked() {
                    match self.store.search(&self.search_query) {
                        Ok(res) => self.results = res,
                        Err(e) => eprintln!("Error al buscar: {}", e),
                    }
                }
            });

            ui.separator();

            // Resultados de archivos
            egui::ScrollArea::vertical().show(ui, |ui| {
                for file in &self.results {
                    ui.horizontal(|ui| {
                        ui.label(&file.name);
                        ui.label(format!("Etiquetas: {:?}", file.tags));
                    });
                }
            });

            ui.separator();

            // Mostrar tags
            ui.group(|ui| {
                ui.label("Lista de Tags:");
                for tag in &self.tags {
                    ui.label(format!("- {}", tag));
                }
            });

            ui.separator();

            // Mostrar categorías
            ui.group(|ui| {
                ui.label("Lista de Categorías:");
                for cat in &self.categories {
                    ui.label(format!("- {}", cat));
                }
            });
        });

        // Tag Manager
        if self.show_tag_manager {
            egui::Window::new("Tag Manager")
                .open(&mut self.show_tag_manager)
                .show(ctx, |ui| {
                    ui.heading("Administrar Tags");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_tag);
                        if ui.button("➕ Add").clicked() && !self.new_tag.trim().is_empty() {
                            let _ = self.tag_store.insert_tag(&self.new_tag);
                            self.new_tag.clear();
                            self.tags = self.tag_store.get_tags().unwrap_or_default();
                        }
                    });
                    ui.separator();
                    for tag in self.tags.clone() {
                        ui.horizontal(|ui| {
                            if let Some((original, nuevo)) = &mut self.edit_tag {
                                if original == &tag {
                                    ui.text_edit_singleline(nuevo);
                                    if ui.button("💾 Save").clicked() {
                                        let _ = self.tag_store.update_tag(original, nuevo);
                                        self.edit_tag = None;
                                        self.tags = self.tag_store.get_tags().unwrap_or_default();
                                    }
                                    if ui.button("❌ Cancel").clicked() {
                                        self.edit_tag = None;
                                    }
                                    return;
                                }
                            }

                            ui.label(&tag);
                            if ui.button("✏️ Edit").clicked() {
                                self.edit_tag = Some((tag.clone(), tag.clone()));
                            }
                            if ui.button("🗑 Delete").clicked() {
                                let _ = self.tag_store.delete_tag(&tag);
                                self.tags = self.tag_store.get_tags().unwrap_or_default();
                            }
                        });
                    }
                });
        }

        // Category Manager
        if self.show_category_manager {
            egui::Window::new("Category Manager")
                .open(&mut self.show_category_manager)
                .show(ctx, |ui| {
                    ui.heading("Administrar Categorías");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_category);
                        if ui.button("➕ Add").clicked() && !self.new_category.trim().is_empty() {
                            // Crear carpeta física
                            let path = Path::new(&self.root_path).join(&self.new_category);
                            if !path.exists() {
                                if let Err(e) = fs::create_dir_all(&path) {
                                    eprintln!("Error creando carpeta de categoría: {}", e);
                                }
                            }

                            let _ = self.categories_store.insert_category(&self.new_category);
                            self.new_category.clear();
                            self.categories = self.categories_store.get_categories().unwrap_or_default();
                        }
                    });
                    ui.separator();
                    for cat in self.categories.clone() {
                        ui.horizontal(|ui| {
                            if let Some((original, nuevo)) = &mut self.edit_category {
                                if original == &cat {
                                    ui.text_edit_singleline(nuevo);
                                    if ui.button("💾 Save").clicked() {
                                        let _ = self.categories_store.update_category(original, nuevo);
                                        self.edit_category = None;
                                        self.categories = self.categories_store.get_categories().unwrap_or_default();
                                    }
                                    if ui.button("❌ Cancel").clicked() {
                                        self.edit_category = None;
                                    }
                                    return;
                                }
                            }

                            ui.label(&cat);
                            if ui.button("✏️ Edit").clicked() {
                                self.edit_category = Some((cat.clone(), cat.clone()));
                            }
                            if ui.button("🗑 Delete").clicked() {
                                // Borrar carpeta física
                                let path = Path::new(&self.root_path).join(&cat);
                                if path.exists() {
                                    if let Err(e) = fs::remove_dir_all(&path) {
                                        eprintln!("Error eliminando carpeta: {}", e);
                                    }
                                }

                                let _ = self.categories_store.delete_category(&cat);
                                self.categories = self.categories_store.get_categories().unwrap_or_default();
                            }
                        });
                    }
                });
        }

				// Item Manager
        if self.show_item_manager {
					egui::Window::new("Item Manager")
					.open(&mut self.show_item_manager)
					.show(ctx, |ui| {
						ui.heading("Agregar Item");

						// Seleccionar categoría
						egui::ComboBox::from_label("Categoría")
						.selected_text(self.selected_category.clone().unwrap_or("None".into()))
						.show_ui(ui, |ui| {
							for cat in &self.categories {
								if ui.selectable_label(
									Some(cat.clone()) == self.selected_category,
									cat
								).clicked() {
									self.selected_category = Some(cat.clone());
								}
							}
						});

						// Nombre del item
						ui.horizontal(|ui| {
							ui.label("Nombre:");
							ui.text_edit_singleline(&mut self.new_item_name);
						});

						// Seleccionar tags existentes o crear
						ui.label("Tags:");
						for tag in &self.tags {
							let mut selected = self.selected_tags.contains(tag);
							if ui.checkbox(&mut selected, tag).clicked() {
								if selected {
									self.selected_tags.push(tag.clone());
								} 
								else {
									self.selected_tags.retain(|t| t != tag);
								}
							}
						}

						// Crear tag nuevo inline
						let mut new_tag_temp = String::new();
						ui.horizontal(|ui| {
							ui.text_edit_singleline(&mut new_tag_temp);
							if ui.button("➕ Add Tag").clicked() && !new_tag_temp.trim().is_empty() {
								let _ = self.tag_store.insert_tag(&new_tag_temp);
								self.tags = self.tag_store.get_tags().unwrap_or_default();
								self.selected_tags.push(new_tag_temp.clone());
								new_tag_temp.clear();
							}
						});

						// Seleccionar archivo de datos
						if ui.button("Seleccionar archivo de datos").clicked() {
							if let Some(path) = FileDialog::new().pick_file() {
								self.item_file_path = Some(path.display().to_string());
							}
						}

						if let Some(path) = &self.item_file_path {
							ui.label(format!("Archivo seleccionado: {}", path));
						}

						// Seleccionar imagen
						if ui.button("Seleccionar imagen de referencia").clicked() {
							if let Some(path) = FileDialog::new().pick_file() {
								self.item_image_path = Some(path.display().to_string());
							}
						}

						if let Some(path) = &self.item_image_path {
							ui.label(format!("Imagen seleccionada: {}", path));
						}

						// Botón guardar item
						if ui.button("Guardar Item").clicked() {
							if let Some(category) = &self.selected_category {
								if !self.new_item_name.trim().is_empty() &&
									self.item_file_path.is_some() &&
									self.item_image_path.is_some()
								{
									let item_path = Path::new(&self.root_path).join(category).join(&self.new_item_name);
									if let Err(e) = fs::create_dir_all(&item_path) {
										eprintln!("Error creando carpeta del item: {}", e);
									}

									// Copiar archivos seleccionados a la carpeta del item
									let _ = fs::copy(
										self.item_file_path.as_ref().unwrap(),
										item_path.join("data.txt") // o mantener el nombre original
									);

									let _ = fs::copy(
										self.item_image_path.as_ref().unwrap(),
										item_path.join("image.png") // o mantener la extensión
									);

									// Aquí podrías guardar los datos del item en la DB si quieres
									// por ejemplo con tabla items(id, name, category, tags_json, file_path, image_path)

									self.new_item_name.clear();
									self.selected_tags.clear();
									self.item_file_path = None;
									self.item_image_path = None;
								}
							}
						}
					});
				}

        if path_changed {
            self.save_config();
            self.sync_categories_with_fs();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions::default();

    // 🔹 Dos conexiones independientes para evitar moves
    let conn1 = rusqlite::Connection::open("files.db")?;
    let conn2 = rusqlite::Connection::open("files.db")?;

    let store = Store::new("files.db")?;

    let tag_store = TagStore::new(conn1);
    tag_store.init()?; // crear tabla de tags

    let categories_store = CategoryStore::new(conn2);
    categories_store.init()?; // crear tabla de categorías

    eframe::run_native(
        "Administrador de Archivos VRC",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new(store, tag_store, categories_store)))),
    )?;

    Ok(())
}
