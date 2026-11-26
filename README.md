
Retro3D & Math Visualizer (Rust)

Motor ligero y entorno de experimentación gráfica en Rust diseñado para el renderizado de escenarios estilo wireframe retrofuturista (estética Alien / Blade Runner), visualización de ecuaciones paramétricas y simulación en tiempo real de sistemas dinámicos (como el atractor de Lorenz).

[dependencies]
# Renderizado 3D ligero y gestión de ventanas/eventos
three-d = { version = "0.17", features = ["egui-gui"] }

# Álgebra lineal rápida (vectores, matrices, cuaterniones)
glam = "0.29"

# Paralelismo en CPU para cálculo masivo de partículas y trayectorias
rayon = "1.10"

# Generación de números aleatorios para condiciones iniciales
rand = "0.8"

# Serialización para guardar/cargar presets de ecuaciones y rutas de cámara
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"



retro3d-visualizer/
├── Cargo.toml                  # Manifiesto de dependencias y configuración del paquete
├── README.md                   # Documentación del proyecto
├── assets/                     # Recursos estáticos
│   ├── shaders/                # Shaders personalizados en WGSL/GLSL (ej. efecto CRT/bloom)
│   └── fonts/                  # Tipografías monoespaciadas para HUD estilo terminal
├── src/
│   ├── main.rs                 # Punto de entrada, bucle principal y gestión de ventana
│   ├── app.rs                  # Estado global de la aplicación y cambio de modos
│   ├── camera/
│   │   ├── mod.rs              # Controlador de cámara (orbital y recorrido libre tipo vuelo)
│   │   └── path.rs             # Rutas preprogramadas para demos automáticos
│   ├── math/
│   │   ├── mod.rs              # Utilidades de integración numérica (Euler, Runge-Kutta 4)
│   │   ├── lorenz.rs           # Sistema de ecuaciones del atractor de Lorenz
│   │   └── parametric.rs       # Generador de mallas y superficies matemáticas
│   ├── render/
│   │   ├── mod.rs              # Configuración de contexto gráfico y pipelines
│   │   ├── wireframe.rs        # Renderizado de líneas vectoriales y grillas retro
│   │   └── postprocess.rs      # Efectos visuales (scanlines, fósforo verde, resplandor)
│   └── ui/
│       └── mod.rs              # Paneles de control en tiempo real (sliders y telemetría)
└── examples/                   # Demos independientes listos para ejecutar
    ├── lorenz_attractor.rs     # Simulación aislada del atractor de Lorenz
    └── wireframe_trench.rs     # Recorrido de terreno vectorial estilo años 80

# Compilación
cargo run --release

# Ejecución

cargo run --release --example lorenz_attractor
cargo run --release --example wireframe_trench

# Ejecutar en navegador web

trunk serve --release



Opciones de Operación y Controles


Argumentos de línea de comandos (CLI)

Puedes iniciar la aplicación directamente en un modo específico:
• --mode lorenz — Inicia en el simulador de atractores caóticos.
• --mode terrain — Inicia el recorrido vectorial estilo wireframe retro.
• --mode equation — Inicia el graficador de funciones z = f(x, y).
• --preset <archivo.json> — Carga parámetros iniciales personalizados.

Controles Interactivos


Acción	Tecla / Ratón
Rotar cámara orbital	Clic Izquierdo + Arrastrar
Desplazar cámara (Pan)	Clic Derecho + Arrastrar
Zoom	Rueda del ratón
Vuelo libre (Modo recorrido)	W, A, S, D + Q/E (Subir/Bajar)
Pausar / Reanudar simulación	Espacio
Reiniciar trayectoria / cámara	R
Ocultar / Mostrar interfaz (HUD)	H
Alternar filtro CRT / Fósforo	F1


Panel de Parámetros en Tiempo Real (UI)

Al presionar H se despliega el panel lateral donde puedes ajustar sin recompilar:
• Constantes del sistema (ej. Lorenz): Deslizadores dinámicos para σ (Sigma), ρ (Rho) y β (Beta), además del paso de tiempo (Δ t) y cantidad de partículas simultáneas.
• Estética Visual: Selector de paleta monocromática (Verde Fósforo #00FF41, Ámbar #FFB000, Cian Vectorial #00FFFF), grosor de línea e intensidad de brillo.
• Exportación: Botón para guardar el estado actual de parámetros en formato JSON o capturar una imagen limpia sin interfaz.
