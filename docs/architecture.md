# Arquitectura de OuroborosDB

Este documento explica como piensa el motor internamente para que no haga falta reconstruirlo leyendo el codigo.

## Componentes

- `OuroborosConfig`: define `data_size` y `max_records`.
- `OuroborosDB`: administra el archivo principal, el cursor y la fase activa.
- `RecordIndex`: direccion fisica de un slot.
- `PhaseBit`: bit almacenado en cada slot para distinguir vueltas del anillo.

## Layout fisico

Cada slot ocupa `record_size = data_size + 1` bytes:

```text
offset = record_index * record_size

+------------+----------------------+
| phase: u8  | payload: [u8; N]     |
+------------+----------------------+
```

El motor expone solo el payload. El `phase bit` queda reservado para recuperacion.

## Modelo del anillo

La base se comporta como un buffer circular persistente:

- El cursor apunta al siguiente slot que usara `append`.
- `append` escribe `phase + payload` en ese slot.
- Luego incrementa el cursor.
- Si el cursor llega a `max_records`, vuelve a `0` y se alterna la fase.

Consecuencia importante: el mismo `RecordIndex` puede representar contenido distinto a lo largo del tiempo.

## Por que existe el phase bit

Si el motor solo guardara payloads, al reiniciar no podria distinguir facilmente entre:

- un anillo nunca completado,
- un anillo lleno,
- un anillo que ya dio una o mas vueltas.

El `phase bit` resuelve esa ambiguedad. Los slots escritos en la vuelta actual comparten una fase;
los slots que aun pertenecen a la vuelta anterior conservan la otra. Eso crea una frontera visible
entre regiones del archivo.

## Recuperacion en apertura

`OuroborosDB::open` sigue este flujo:

1. Abre o crea el archivo principal.
2. Si esta vacio, lo preasigna al tamano exacto del anillo.
3. Verifica que el tamano existente coincida con `max_records * record_size`.
4. Llama a `recover_state` para encontrar el siguiente punto de escritura.

`recover_state` lee el `phase bit` del primer y del ultimo slot:

- Si ambos son iguales, el motor interpreta que no hay frontera observable y reinicia en `RecordIndex(0)` con la fase alternada.
- Si son distintos, existe una transicion de fase dentro del archivo. El motor la encuentra con busqueda binaria y usa ese indice como siguiente cursor.

El coste de esta recuperacion es `O(log N)` respecto a `max_records`.

## Concurrencia

El motor usa I/O posicional sobre `File`:

- Unix: `read_exact_at` y `write_all_at`.
- Windows: `seek_read` y `seek_write` envueltos en bucles para completar la operacion.

Como `read` no depende de un cursor compartido del archivo, puede recibir `&self`.
Eso habilita un patron MRSW si la aplicacion envuelve el motor en `Arc<RwLock<_>>`.

La libreria no ofrece:

- multiples escritores internos,
- snapshots,
- transacciones,
- coordinacion distribuida.

## Metadata sidecar

La configuracion estructural vive en `<db>.meta` como JSON. Ese archivo congela:

- `data_size`
- `max_records`
- `version`

La apertura futura ignora el entorno si el `.meta` existe. Eso evita reinterpretar el archivo principal con otra geometria.

## Decisiones y tradeoffs

- Payload fijo: simplifica offsets, preasignacion y recuperacion.
- Sin timestamps ni encabezados extra: minimiza overhead por slot.
- `RecordIndex` fisico: la API es simple, pero el consumidor debe modelar por separado su nocion de orden cronologico si la necesita.
- `update` preserva fase: permite corregir contenido sin corromper el algoritmo de reapertura.