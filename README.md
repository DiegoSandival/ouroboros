# OuroborosDB

OuroborosDB es un motor de almacenamiento circular orientado a payloads de tamano fijo.
Escribe registros en un anillo sobre un archivo preasignado y, cuando llega al final,
vuelve al inicio y sobrescribe los slots mas antiguos.

La libreria esta pensada para telemetria, buffers de eventos, logs compactos y escenarios
en los que importa mas conservar una ventana reciente de datos que el historial completo.

## Que problema resuelve

- Escritura secuencial sobre disco con capacidad fija.
- Reapertura rapida tras reinicio sin escanear todo el archivo.
- Lecturas concurrentes a nivel de API gracias a I/O posicional.
- Configuracion estructural inmutable mediante un archivo `.meta`.

## Cuando usarla

- Telemetria de alta frecuencia.
- Buffers persistentes para dispositivos con almacenamiento acotado.
- Colas de eventos donde solo importa la ventana mas reciente.
- Persistencia de blobs binarios, JSON serializado o estructuras empaquetadas a bytes.

## Cuando no usarla

- Cuando necesitas registros de tamano variable.
- Cuando necesitas indices secundarios, consultas complejas o borrado selectivo.
- Cuando debes preservar el historial completo sin sobrescritura.
- Cuando necesitas transacciones o multiples escritores coordinados por la libreria.

## Modelo mental

Piensa en la base como un arreglo circular de `max_records` slots. Cada slot contiene:

```text
[ 1 byte phase bit | N bytes payload ]
```

- `N` es `data_size`.
- `RecordIndex` identifica un slot fisico, no una posicion cronologica global.
- `next_write_index()` expone el slot donde caera el siguiente `append`.
- `append` escribe en el cursor actual y devuelve el slot usado.
- Cuando el cursor completa una vuelta, vuelve a `0` y el `phase bit` cambia de `0` a `1` o de `1` a `0`.
- Esa alternancia permite recuperar en que punto del anillo estaba la siguiente escritura usando busqueda binaria en vez de escanear todo el archivo.

## Invariantes importantes

- Todos los payloads deben medir exactamente `config.data_size` bytes.
- La capacidad fisica queda fijada por `config.max_records`.
- El archivo `<db>.meta` es la fuente de verdad estructural al reabrir la base.
- `append` puede sobrescribir datos antiguos cuando el anillo da una vuelta completa.
- `update` modifica el payload de un slot sin tocar su `phase bit`.
- `read(RecordIndex(i))` lee el slot fisico `i`; no implica que sea el registro mas nuevo ni el mas viejo.

## Archivos en disco

Abrir una base usa dos archivos:

- `datos.db`: archivo principal preasignado con `max_records * (data_size + 1)` bytes.
- `datos.db.meta`: sidecar JSON con `data_size`, `max_records` y la version del formato.

Si el archivo principal existe pero su tamano no coincide con los metadatos, la apertura falla.

## Flujo de uso recomendado

```rust
use ouroboros_db::{OuroborosConfig, OuroborosDB, RecordIndex, Result};

fn main() -> Result<()> {
    let config = OuroborosConfig::load_or_init("telemetria.db")?;
    let mut db = OuroborosDB::open("telemetria.db", config.clone())?;

    assert_eq!(db.next_write_index().0, 0);

    let payload = [7u8; 96];
    let slot = db.append(&payload)?;

    let bytes = db.read(slot)?;
    assert_eq!(bytes[0], 7);

    db.update(RecordIndex(0), &[9u8; 96])?;
    Ok(())
}
```

## Configuracion

`OuroborosConfig::load_or_init` tiene dos comportamientos:

1. Si `<db>.meta` ya existe, lee la configuracion desde ese archivo e ignora el entorno.
2. Si no existe, intenta crear la configuracion leyendo estas variables:

```env
OUROBOROS_DATA_SIZE=96
OUROBOROS_MAX_RECORDS=1000000
```

Esas dimensiones quedan congeladas en el `.meta` para futuras aperturas.

## Concurrencia

La libreria expone este contrato:

- `read(&self, ...)` permite lecturas concurrentes.
- `append(&mut self, ...)` y `update(&mut self, ...)` requieren acceso exclusivo.

La sincronizacion multi-hilo no la implementa el motor por si mismo. El patron esperado es
envolver `OuroborosDB` en algo como `Arc<RwLock<_>>` desde la aplicacion. El ejemplo completo
esta en `examples/basico.rs`.

## Estado operacional minimo

La API publica expone `next_write_index()` para consultar el cursor actual del anillo.
Eso permite instrumentar la libreria o razonar sobre el siguiente slot de escritura sin
abrir el codigo ni acceder a campos privados.

## Recuperacion tras reinicio

Al abrir la base, `OuroborosDB::open` reconstruye el punto de escritura leyendo bits de fase
del archivo y aplicando una busqueda binaria. Eso le permite reanudar en tiempo `O(log N)`
respecto a `max_records`.

Casos practicos:

- Base nueva: inicializa el archivo y comienza en `RecordIndex(0)`.
- Base llena sin vuelta observable: sigue en `RecordIndex(0)` con fase invertida.
- Base con mezcla de fases: busca el punto donde cambia la fase y reanuda ahi.

## Errores que debes esperar

- `InvalidDataSize`: el payload no mide `data_size`.
- `IndexOutOfBounds`: intentaste leer o actualizar un slot fuera de `0..max_records`.
- `CorruptedMetadata`: falta o no se puede interpretar el archivo `.meta`.
- `ConfigError`: faltan variables de entorno, el archivo fisico no coincide con la configuracion o hay un problema inicializando la base.

## Documentacion complementaria

- `docs/architecture.md`: modelo interno, algoritmo de recuperacion y decisiones de diseno.
- `docs/spec.md`: comportamiento observable de la API e invariantes formales.
- `docs/faq.md`: respuestas cortas para uso comun y troubleshooting.
- `examples/basico.rs`: demo completa con escritura, reapertura y concurrencia.

## Desarrollo local

Ejecuta las pruebas con:

```bash
cargo test
```

Si todavia no publicas la libreria en `crates.io`, puedes consumirla por `path` o por Git,
segun donde viva el repositorio real.
