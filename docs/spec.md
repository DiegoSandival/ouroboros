# Especificacion observable

Este documento describe el contrato externo de OuroborosDB.

## Tipos publicos

### `OuroborosConfig`

- `data_size: usize`: tamano exacto del payload de usuario.
- `max_records: u32`: cantidad fija de slots del anillo.
- `record_size() -> u64`: devuelve `data_size + 1` por el `phase bit`.

### `RecordIndex`

- Representa un slot fisico dentro de `0..max_records`.
- No garantiza orden temporal.

## `OuroborosConfig::load_or_init(db_path)`

Precondiciones:

- Si no existe `<db_path>.meta`, deben existir `OUROBOROS_DATA_SIZE` y `OUROBOROS_MAX_RECORDS` en el entorno o en `.env`.

Postcondiciones:

- Si la metadata ya existia, devuelve exactamente esa configuracion.
- Si no existia, crea `<db_path>.meta` y persiste la configuracion calculada.

Errores:

- `ConfigError` si faltan variables, no son enteros validos o no se puede serializar.
- `CorruptedMetadata` si el sidecar existe pero no se puede parsear.
- `Io` si falla la lectura o escritura del sidecar.

## `OuroborosDB::open(path, config)`

Precondiciones:

- `config` debe coincidir con la geometria real del archivo si este ya existe.

Postcondiciones:

- Si el archivo principal estaba vacio, queda preasignado a `max_records * record_size()` bytes.
- El motor queda listo para `append`, `read` y `update`.
- El cursor interno apunta al siguiente slot a escribir.

Errores:

- `ConfigError` si el tamano del archivo no coincide con la configuracion.
- `Io` si falla la apertura, preasignacion o lectura del archivo.

## `next_write_index(&self)`

Postcondiciones:

- Devuelve el slot fisico donde el proximo `append` escribira.
- No modifica el estado del motor.

Observaciones:

- Expone el cursor operativo actual.
- No garantiza por si solo que pueda reconstruirse el ultimo registro cronologico sin informacion adicional.

## `append(&mut self, data)`

Precondiciones:

- `data.len() == config.data_size`.

Postcondiciones:

- El payload se escribe en el slot apuntado por el cursor al inicio de la llamada.
- Devuelve ese `RecordIndex`.
- El cursor avanza una posicion.
- Si el cursor completa una vuelta, vuelve a `0` y se alterna la fase activa.

Errores:

- `InvalidDataSize` si el payload no coincide con `data_size`.
- `Io` si falla la escritura.

## `read(&self, index)`

Precondiciones:

- `index.0 < max_records`.

Postcondiciones:

- Devuelve una copia del payload del slot fisico indicado.
- No modifica cursor ni fase.

Errores:

- `IndexOutOfBounds` si el indice esta fuera de rango.
- `Io` si falla la lectura.

## `update(&mut self, index, data)`

Precondiciones:

- `index.0 < max_records`.
- `data.len() == config.data_size`.

Postcondiciones:

- Reemplaza solo el payload del slot.
- Conserva el `phase bit` original del slot.
- No cambia cursor ni fase activa del motor.

Errores:

- `IndexOutOfBounds` si el indice esta fuera de rango.
- `InvalidDataSize` si el payload no coincide con `data_size`.
- `Io` si falla la lectura del `phase bit` o la escritura del slot.

## Garantias

- La geometria del anillo se mantiene estable a traves del `.meta`.
- La reapertura no necesita escanear todo el archivo.
- Las lecturas pueden ejecutarse con `&self` porque usan offsets explicitos.

## No garantizado

- Orden cronologico accesible por API.
- Deteccion de registros nunca escritos frente a payloads de ceros.
- Multiples escritores coordinados internamente.
- Durabilidad extra mediante `fsync` explicito por operacion.
- Compatibilidad hacia atras entre futuras versiones del formato mas alla del campo `version` en metadata.