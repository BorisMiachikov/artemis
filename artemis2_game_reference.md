# 🚀 Артемида 2 — Игровой справочник по миссии
> Справочный документ для разработки игры «Полёт на Луну» на Rust

---

## 1. ОБЗОР МИССИИ

**Артемида II (Artemis II)** — первый пилотируемый облёт Луны в рамках программы NASA «Артемида», первый пилотируемый полёт за пределы низкой орбиты Земли со времён миссии «Аполлон-17» (1972).

| Параметр | Значение |
|---|---|
| Даты миссии | 1–11 апреля 2026 |
| Старт | LC-39B, Космический центр Кеннеди, Флорида |
| Время старта (EDT) | 18:35, 1 апреля 2026 |
| Продолжительность | ~10 суток |
| Тип миссии | Пилотируемый облёт Луны (без посадки) |
| Название корабля | Orion «Integrity» |
| Приводнение | 10 апреля 2026, Тихий океан, побережье Сан-Диего |

---

## 2. ЭКИПАЖ

| Позиция | Имя | Агентство |
|---|---|---|
| Командир | Рид Уайзман (Reid Wiseman) | NASA |
| Пилот | Виктор Гловер (Victor Glover) | NASA |
| Специалист миссии | Кристина Кох (Christina Koch) | NASA |
| Специалист миссии | Джереми Хансен (Jeremy Hansen) | CSA (Канада) |

> Джереми Хансен — первый не-американец, летевший на Луну.

---

## 3. ТРАЕКТОРИЯ И КЛЮЧЕВЫЕ СОБЫТИЯ

```
СТАРТ (LC-39B, KSC)
       │
       ▼ ~+8 мин
   Выход на орбиту (LEO)
       │
       ▼ ~+1-2 сут
   Maneuver — Trans-Lunar Injection (TLI)
       │
       ▼ ~+4-5 сут
   Максимальное удаление от Земли:
   252 760 миль (406 840 км) — рекорд для людей
       │
       ▼ Ближайший подлёт к Луне:
   6 556 км (4 070 миль) от поверхности
       │
       ▼ ~+9-10 сут
   Возвращение / вхождение в атмосферу
       │
       ▼
   ПРИВОДНЕНИЕ (Тихий океан, у берегов Сан-Диего)
   Скорость при ударе: 27 км/ч (на 11 парашютах)
```

**Общий путь:** 694 481 миля (1 117 680 км) — полный маршрут

---

## 4. РАКЕТА-НОСИТЕЛЬ — SLS Block 1

**Space Launch System (SLS)** — сверхтяжёлая ракета-носитель NASA.

| Характеристика | Значение |
|---|---|
| Конфигурация | Block 1 (для Артемиды II) |
| Высота | 98 м (322 фута) — выше Статуи Свободы |
| Стартовая масса | 2 608 000 кг (5,75 млн фунтов) |
| Тяга при старте | 39 МН (8 800 000 фунтов-сил) |
| Полезная нагрузка на TLI | 27 т |

### Ступени SLS Block 1

**Центральная ступень (Core Stage)**
- Высота: 64,6 м (212 футов)
- Диаметр: 8,4 м (27,6 фута)
- Двигатели: 4 × RS-25 (на жидком кислороде и жидком водороде)

**Твёрдотопливные ускорители (SRB)**
- Количество: 2 (по бокам от центральной ступени)
- Масса каждого: 726 000 кг (1,6 млн фунтов)
- Тяга каждого: ~16 МН (3 600 000 фунтов-сил)
- 5 секций (на 25% больше топлива, чем у шаттловских SRB)

**Верхняя ступень (ICPS — Interim Cryogenic Propulsion Stage)**
- Двигатель: 1 × RL-10
- Используется для разгона к Луне (TLI)

---

## 5. КОСМИЧЕСКИЙ КОРАБЛЬ — ORION

| Характеристика | Значение |
|---|---|
| Экипаж | 4 человека |
| Командный модуль (диаметр) | 5 м |
| Общая длина с ESM | ~9 м |
| Теплозащитный экран | Абляционный, диаметр 5 м |
| Скорость входа в атмосферу | ~11 км/с (40 000 км/ч) |

**European Service Module (ESM)** — европейский сервисный модуль (разработка ESA/Airbus):
- Тяговый двигатель: AJ10 (21,1 кН тяги)
- Солнечные панели: 4 панели (11 кВт мощности)
- Топливо: NTO + MMH
- Запас воды, кислорода и азота для экипажа

---

## 6. СТАРТОВЫЙ КОМПЛЕКС 39B (LC-39B)

**Локация:** Космический центр Кеннеди (KSC), Флорида

- Ранее использовался для программы «Спейс Шаттл»
- Мобильная пусковая платформа (Mobile Launcher) высотой ~105 м
- Огнезащитный жёлоб (Flame Trench) с системой водяного охлаждения
- Товарно-транспортный гусеничный транспортёр (Crawler-Transporter)

---

## 7. РЕЗУЛЬТАТЫ МИССИИ

- Рекордное удаление людей от Земли: **252 760 миль** (побит рекорд Аполлон-13)
- Успешное тестирование систем жизнеобеспечения в глубоком космосе
- Значительно уменьшены проблемы с тепловым экраном, выявленные на Артемиде I
- Подтверждена готовность к Артемиде III (высадка на Луну)

---

## 8. 3D-АССЕТЫ ДЛЯ ИГРЫ

### Ракета SLS и корабль Orion

| Ассет | Платформа | Стоимость | Форматы | Ссылка |
|---|---|---|---|---|
| NASA SLS Rocket Block 1 | Sketchfab (AllThingsSpace) | Бесплатно | GLTF/GLB | [Ссылка](https://sketchfab.com/3d-models/nasa-sls-rocket-block-1-81bb895c07d04e788eb001abd4890c46) |
| SLS and Payloads (коллекция) | Sketchfab (ArcturusVFX) | Бесплатно | GLTF/GLB | [Ссылка](https://sketchfab.com/arcturusvfx/collections/sls-and-payloads-0f46a40924094a2fbcfab92c2989fff3) |
| SLS + Orion (полная сборка) | Sketchfab / BlenderKit | Бесплатно | BLEND | [BlenderKit](https://www.blenderkit.com/asset-gallery-detail/7e77522e-e178-44f1-8d6b-83711bb58f11/) |
| NASA 3D Resources (все модели) | NASA Science | Бесплатно | OBJ, STL | [NASA 3D](https://science.nasa.gov/3d-resources/) |

### Луна и лунная поверхность

| Ассет | Платформа | Стоимость | Ссылка |
|---|---|---|---|
| NASA CGI Moon Kit (официальный) | Sketchfab (Thomas Flynn) | Бесплатно | [Ссылка](https://sketchfab.com/3d-models/nasa-cgi-moon-kit-1c496b3b57304526b5b9d1cf9c1087fc) |
| Moon Surface (8K текстуры, кратеры) | Sketchfab | Бесплатно | [Ссылка](https://sketchfab.com/3d-models/moon-surface-489dac88bfa4453fadac38d03fcd1de9) |
| NASA Lunar Reconnaissance Orbiter 3D | NASA SVS | Бесплатно | [NASA SVS](https://svs.gsfc.nasa.gov/14959/) |
| Коллекция Луна — теги | Sketchfab | Бесплатно | [Теги](https://sketchfab.com/tags/moon) |

### Земля

| Ассет | Платформа | Стоимость | Ссылка |
|---|---|---|---|
| Earth (реалистичная) | Sketchfab (Akshat) | Бесплатно | [Ссылка](https://sketchfab.com/3d-models/earth-41fc80d85dfd480281f21b74b2de2faa) |
| NASA Earth Resources | NASA 3D | Бесплатно | [NASA 3D](https://nasa3d.arc.nasa.gov/search/earth) |

### Солнечная система

| Ассет | Платформа | Стоимость | Ссылка |
|---|---|---|---|
| Solar System (Real Scale, 2K) | Sketchfab (FyorDev) | Бесплатно | [Ссылка](https://sketchfab.com/3d-models/solar-system-real-scale-2k-textures-febde2b6e3f64b06965620fd3ddc97c2) |
| Solar System (все планеты) | Sketchfab (dannzjs) | Бесплатно | [Ссылка](https://sketchfab.com/3d-models/solar-system-96e701793bca476fac958985ee256a99) |

### Стартовый комплекс (LC-39B)

| Ассет | Платформа | Стоимость | Ссылка |
|---|---|---|---|
| NASA Kennedy Space Center 39B | Sketchfab (SQUIR3D) | Платно | [Ссылка](https://sketchfab.com/3d-models/nasa-kennedy-space-center-39b-5cc7bcd962bb462ba03043a1efe03ffe) |
| Launch Complex 39-A (альтернатива) | Sketchfab | Платно | [Ссылка](https://sketchfab.com/3d-models/kennedy-space-center-launch-complex-39-a-65e1814a6f0f432cb090ba3f5293643f) |
| Kennedy Space Center 39B (скачать) | Deep3DSea | Платно | [deep3dsea.com](https://deep3dsea.com/downloads/nasa-kennedy-space-center-39b/) |

---

## 9. ЗВУКОВЫЕ АССЕТЫ

### Запуск и двигатели

| Звук | Платформа | Стоимость | Ссылка |
|---|---|---|---|
| Space Shuttle Launch (реалистичный) | Freesound (CGEffex) | Бесплатно | [Ссылка](https://freesound.org/people/CGEffex/sounds/93078/) |
| Rocket Launch SFX (подборка) | Uppbeat | Бесплатно | [Ссылка](https://uppbeat.io/sfx/category/rocket/rocket-launch) |
| Space Shuttle Launch (HD видео, аудио) | YouTube | Бесплатно | [YouTube](https://www.youtube.com/watch?v=OnoNITE-CLc) |

### Переговоры астронавтов / NASA comm

| Звук | Платформа | Стоимость | Ссылка |
|---|---|---|---|
| NASA Audio Highlight Reels (архив) | Internet Archive | Бесплатно | [archive.org](https://archive.org/details/NasaAudioHighlightReels) |
| Space Shuttle Mission Sounds (плейлист) | SoundCloud (NASA) | Бесплатно | [SoundCloud](https://soundcloud.com/nasa/sets/space-shuttle-mission-sounds) |
| Radio/NASA/Space Comms SFX | ZapSplat | Бесплатно | [ZapSplat](https://www.zapsplat.com/music/radio-nasa-space-rocket-launch-comms-communication-satellite-human-voice-transmission-contact/) |

### Коллекции NASA SFX

| Звук | Платформа | Стоимость | Ссылка |
|---|---|---|---|
| NASA Sound Effects (478 звуков) | Pond5 | От $2 | [Pond5](https://www.pond5.com/sound-effects/tag/nasa/) |
| NASA Launch Royalty-Free | Pond5 | От $2 | [Pond5](https://www.pond5.com/sound-effects/1/nasa-launch.html) |

---

## 10. ИДЕИ ДЛЯ ИГРОВЫХ МЕХАНИК

На основе реальных данных миссии можно реализовать:

**Предстартовый этап**
- Прохождение проверок систем (T-10 часов)
- Заправка ракеты жидким водородом и кислородом (T-6 часов)
- Погрузка экипажа (T-2 часа)

**Запуск (0–8 минут)**
- SRB горят первые ~2 минуты, затем сбрасываются
- Центральная ступень работает 8 минут, выводит на орбиту
- Разделение ступеней (SEP события)

**Орбита и TLI (1-2 сутки)**
- Проверка систем корабля Orion
- Двигательный манёвр TLI (~360 с горения ICPS)

**Лунный перелёт (4-5 суток)**
- Навигация в открытом космосе
- Радиационные пояса Ван Аллена (риск)
- Вид Земли с 400 000 км

**Облёт Луны**
- Максимальное сближение: 6 556 км
- Гравитационный манёвр у Луны
- Фотографирование поверхности

**Возвращение и вход в атмосферу**
- Скорость входа: 40 000 км/ч
- Тепловой экран нагревается до ~2760°C
- Развёртывание 11 парашютов
- Приводнение в Тихом океане

---

## 11. ИСТОЧНИКИ

- [NASA Artemis II Official](https://www.nasa.gov/mission/artemis-ii/)
- [Wikipedia — Artemis II](https://en.wikipedia.org/wiki/Artemis_II)
- [SLS Technical Specs — NASA](https://www.nasa.gov/reference/space-launch-system/)
- [Britannica — Artemis II](https://www.britannica.com/topic/Artemis-II)
- [NASA 3D Resources](https://science.nasa.gov/3d-resources/)
- [Sketchfab — SLS Block 1](https://sketchfab.com/3d-models/nasa-sls-rocket-block-1-81bb895c07d04e788eb001abd4890c46)
- [NASA Audio Archive](https://archive.org/details/NasaAudioHighlightReels)
- [Freesound Shuttle Launch](https://freesound.org/people/CGEffex/sounds/93078/)

---

*Документ подготовлен: апрель 2026. Для разработки игры на Rust.*
