#let waybill_truck(
  driver: "",
  driving_license: "",
  driving_license_date: "",
  tractor_license: "",
  tractor_license_date: "",
  vehicle: "",
  license_plate: "",
  sts: "",
  category: "",
  date: "",
  company_name: "",
  company_address: "",
  company_inn: "",
  dispatcher_name: "",
  mechanic_name: "",
  medic_name: "",
  medical_exam_time: "07:30",
  departure_time: "",
  return_time: "",
  fuel_brand: "",
  fuel_code: "",
  fuel_issued: "",
  fuel_remain_depart: "",
  fuel_remain_return: "",
  fuel_submitted: "",
  fuel_coeff: "",
  fuel_special: "",
  fuel_engine: "",
  no_date: false,
  no_time: false,
  tasks: ()
) = {
  set page(paper: "a4", flipped: true, margin: 0.5cm)
  set text(font: "DejaVu Sans", size: 7.2pt)

  let field(body, w: auto) = box(width: w, stroke: (bottom: 0.5pt), inset: (bottom: 1pt), outset: (bottom: 1pt))[#body]
  let empty_f(w) = box(width: w, stroke: (bottom: 0.5pt))
  let sub(body) = text(size: 5pt)[#body]
  let h_cell(h, body) = box(height: h, width: 100%, align(center + horizon)[#body])

  // Функция для получения инициалов из полного имени
  let get_initials(name) = {
    if name == "" {
      ""
    } else {
      let parts = name.split(" ")
      if parts.len() >= 2 {
        parts.at(0) + " " + parts.at(1).at(0) + "." + if parts.len() >= 3 { parts.at(2).at(0) + "." } else { "" }
      } else { name }
    }
  }
  
  // Строка подписи: инициалы справа от прочерка (без надписи "подпись")
  let sig_line(label, name, width) = grid(
    columns: (auto, 1fr, auto),
    column-gutter: 3pt,
    align: bottom,
    [#label], [#box(width: 100%, stroke: (bottom: 0.5pt))[]], [#sub[#get_initials(name)]]
  )
  
  let sig_doc_line(label, name, width) = grid(
    columns: (auto, 1fr, auto),
    column-gutter: 3pt,
    align: bottom,
    [#label], [#box(width: 100%, stroke: (bottom: 0.5pt))[]], [#sub[#get_initials(name)]]
  )

  // Функция для отображения даты (пусто если no_date)
  let show_date(d) = {
    if no_date { "" } else { d }
  }
  
  // Функция для отображения времени (пусто если no_time)
  let show_time(t) = {
    if no_time { "" } else { t }
  }
  
  // Функция для отображения даты и времени с одним прочерком
  let show_date_time(d, t) = {
    let has_date = not no_date and d != ""
    let has_time = not no_time and t != ""
    if has_date and has_time {
      d + " " + t
    } else if has_date {
      d
    } else if has_time {
      t
    } else {
      box(width: 100%, stroke: (bottom: 0.5pt))[]
    }
  }

  // Парсим время в часы и минуты
  let parse_time(t) = {
    if t == "" or no_time {
      ("", "")
    } else if t.contains(":") {
      let parts = t.split(":")
      (parts.at(0, default: ""), parts.at(1, default: ""))
    } else if t.contains(".") {
      let parts = t.split(".")
      (parts.at(0, default: ""), parts.at(1, default: ""))
    } else {
      (t, "")
    }
  }

  // Парсим дату в день и месяц
  let parse_date(d) = {
    if d == "" or no_date {
      ("", "")
    } else if d.contains(".") {
      let parts = d.split(".")
      (parts.at(0, default: ""), parts.at(1, default: ""))
    } else if d.contains("-") {
      // ISO формат: 2026-04-15
      let parts = d.split("-")
      if parts.len() >= 3 {
        (parts.at(2, default: ""), parts.at(1, default: ""))
      } else {
        ("", "")
      }
    } else if d.contains("/") {
      let parts = d.split("/")
      (parts.at(0, default: ""), parts.at(1, default: ""))
    } else {
      (d, "")
    }
  }

  let org_line = company_name
  if company_address != "" {
    org_line = org_line + ", " + company_address
  }
  if company_inn != "" {
    org_line = org_line + ", ИНН " + company_inn
  }

  let t1 = tasks.at(0, default: (customer: "", loading_point: "", unloading_point: "", cargo: "", trips: "", distance: "", tons: "", arrival_time: ""))
  let t2 = tasks.at(1, default: (customer: "", loading_point: "", unloading_point: "", cargo: "", trips: "", distance: "", tons: "", arrival_time: ""))
  let t3 = tasks.at(2, default: (customer: "", loading_point: "", unloading_point: "", cargo: "", trips: "", distance: "", tons: "", arrival_time: ""))

  // Парсим время выезда и возврата
  let (dep_hour, dep_min) = parse_time(departure_time)
  let (ret_hour, ret_min) = parse_time(return_time)
  // Парсим дату
  let (day, month) = parse_date(date)

  [
    #rect(width: 100%, height: 100%, stroke: 0.5pt, inset: 10pt)[
      #grid(
        columns: (1fr, 2fr, 1.2fr),
        align: (left, center, right),
        [
          #v(15pt)
        ],
        [
          #text(13pt, weight: "bold")[ПУТЕВОЙ ЛИСТ] #linebreak()
          #text(11pt, weight: "bold")[грузового автомобиля] № #empty_f(2.5cm) #linebreak()
          #v(3pt)
          #let d = if not no_date and date.contains(".") { date.split(".") } else { ("  ", "  ", "    ") }
          #text(size: 10pt, weight: "bold")[#d.at(0) #if d.at(1) == "04" { "апреля" } else { d.at(1) } 20#if d.at(2).len() > 2 { d.at(2).slice(2) } else { d.at(2) } г.]
        ],
        [
          #text(size: 6pt)[
            Типовая межотраслевая форма № 4-С
            Утверждена постановлением Госкомстата России
            от 28.11.97 № 78
          ]
        ]
      )

      #v(5pt)

      #grid(
        columns: (1fr, auto),
        column-gutter: 10pt,
        [
          #grid(
            columns: (auto, 1fr),
            column-gutter: 5pt,
            row-gutter: 4pt,
            align: bottom,
            [Организация], [#field(org_line, w: 100%)
 #align(center)[#sub[наименование, адрес, номер телефона]]],
          )
          #v(4pt)
          #grid(
            columns: (auto, 1fr, auto, 2.2cm, auto, 1.5cm),
            column-gutter: 5pt,
            align: bottom,
            row-gutter: 8pt,
            [Марка автомобиля], [#field(vehicle, w: 100%)], [Гос. номер], [#field(license_plate, w: 100%)], [СТС], [#field(sts, w: 100%)],
            [Водитель], [#field(driver, w: 100%)], [Табельный №], [#empty_f(100%)], [], [],
          )
          #v(4pt)
          #grid(
            columns: (auto, 1.2fr, auto, 0.8fr, auto, 1fr),
            column-gutter: 5pt,
            align: bottom,
            row-gutter: 8pt,
            [Удостоверение №], [#field(driving_license, w: 100%)], [Категория], [#field(category, w: 100%)], [Дата выдачи], [#field(driving_license_date, w: 100%)],
            [Лиценз. карточка], [#empty_f(100%)], [Рег. №], [#empty_f(100%)], [Серия], [#empty_f(100%)],
            [Прицеп 1], [#empty_f(100%)], [Гос. номер], [#empty_f(100%)], [СТС прицепа], [#empty_f(100%)],
            [Прицеп 2], [#empty_f(100%)], [Гос. номер], [#empty_f(100%)], [СТС прицепа], [#empty_f(100%)],
          )
          #v(4pt)
          #grid(
            columns: (auto, 1fr),
            column-gutter: 5pt,
            align: bottom,
            [Сопровождающие лица], [#empty_f(100%)]
          )
        ],
        [
          #grid(
            columns: (auto, auto),
            column-gutter: 5pt,
            [
              #grid(
                columns: (2.2cm, 0.8cm),
                stroke: 0.5pt,
                inset: 3pt,
                align: center,
                [], [*Код*],
                [Режим работы], [],
                [Колонна], [],
                [Бригада], []
              )
            ],
            [
              #align(right)[
                #grid(
                  columns: (1.8cm, 0.7cm, 0.7cm, 0.7cm, 0.7cm, 1.2cm, 1.6cm, 1.8cm),
                  stroke: 0.5pt,
                  inset: 2pt,
                  align: center + horizon,
                  grid.cell(rowspan: 2)[*Работа водителя и ТС*],
                  grid.cell(colspan: 4)[время по графику],
                  grid.cell(rowspan: 2)[нулевой\ проб.км],
                  grid.cell(rowspan: 2)[спидом.\ км],
                  grid.cell(rowspan: 2)[время факт.\ ч, мин.]
                  ,
                  [чис.], [мес.], [ч], [мин],
                  [1], [2], [3], [4], [5], [6], [7], [8],
                  [#h_cell(15pt, "выезд")], [#h_cell(15pt, day)], [#h_cell(15pt, month)], [#h_cell(15pt, dep_hour)], [#h_cell(15pt, dep_min)], [], [], [],
                  [#h_cell(15pt, "возврат")], [#h_cell(15pt, day)], [#h_cell(15pt, month)], [#h_cell(15pt, ret_hour)], [#h_cell(15pt, ret_min)], [], [], [],
                )
              ]
            ]
          )
          #v(4pt)
          #align(right)[
            #grid(
              columns: (0.9cm, 0.8cm, 1.1cm, 1.1cm, 1.1cm, 1cm, 1cm, 1cm, 1.2cm),
              stroke: 0.5pt,
              inset: 2pt,
              align: center + horizon,
              grid.cell(colspan: 9)[*Движение горючего*],
              grid.cell(colspan: 2)[горючее],
              grid.cell(rowspan: 2)[выдано],
              grid.cell(colspan: 2)[остаток при],
              grid.cell(rowspan: 2)[сдано],
              grid.cell(rowspan: 2)[коэфф.],
              grid.cell(colspan: 2)[Время работы],
              [марка], [код], [выезде], [возвр.], [спец.], [двиг.],
              [9], [10], [11], [12], [13], [14], [15], [16], [17],
              [#h_cell(12pt, fuel_brand)], [#h_cell(12pt, fuel_code)], [#h_cell(12pt, fuel_issued)], [#h_cell(12pt, fuel_remain_depart)], [#h_cell(12pt, fuel_remain_return)], [#h_cell(12pt, fuel_submitted)], [#h_cell(12pt, fuel_coeff)], [#h_cell(12pt, fuel_special)], [#h_cell(12pt, fuel_engine)]
            )
          ]
        ]
      )

      #v(0pt)
      #align(center)[*ЗАДАНИЕ ВОДИТЕЛЮ*]
      #grid(
        columns: (0.9fr, 1.5cm, 1fr, 1fr, 2.5cm, 1.2cm, 1.2cm, 1.2cm),
        stroke: 0.5pt,
        inset: 3pt,
        align: center + horizon,
        grid.cell(rowspan: 2)[В чье распоряжение\ (наименование и адрес)],
        grid.cell(rowspan: 2)[время\ приб.],
        grid.cell(colspan: 2)[адрес пункта],
        grid.cell(rowspan: 2)[наименование груза],
        grid.cell(rowspan: 2)[ездок],
        grid.cell(rowspan: 2)[расст.\ км],
        grid.cell(rowspan: 2)[тонн],
        [погрузки], [разгрузки],
        [18], [19], [20], [21], [22], [23], [24], [25],
        [#h_cell(18pt, t1.customer)], [#h_cell(18pt, t1.arrival_time)], [#h_cell(18pt, t1.loading_point)], [#h_cell(18pt, t1.unloading_point)], [#h_cell(18pt, t1.cargo)], [#h_cell(18pt, t1.trips)], [#h_cell(18pt, t1.distance)], [#h_cell(18pt, t1.tons)],
        [#h_cell(18pt, t2.customer)], [#h_cell(18pt, t2.arrival_time)], [#h_cell(18pt, t2.loading_point)], [#h_cell(18pt, t2.unloading_point)], [#h_cell(18pt, t2.cargo)], [#h_cell(18pt, t2.trips)], [#h_cell(18pt, t2.distance)], [#h_cell(18pt, t2.tons)],
        [#h_cell(18pt, t3.customer)], [#h_cell(18pt, t3.arrival_time)], [#h_cell(18pt, t3.loading_point)], [#h_cell(18pt, t3.unloading_point)], [#h_cell(18pt, t3.cargo)], [#h_cell(18pt, t3.trips)], [#h_cell(18pt, t3.distance)], [#h_cell(18pt, t3.tons)],
        grid.cell(colspan: 7, align: right)[*Итого*], []
      )

      #v(6pt)
      #set text(size: 8.2pt)
      
      #grid(
        columns: (1.5fr, 1.5fr, 1fr),
        column-gutter: 15pt,
        [
          Водительское удостоверение проверил, задание выдал,\ выдать горючее #empty_f(1.5cm) литров
          #v(2pt)
          #sig_doc_line("Диспетчер", dispatcher_name, 4cm)
          #v(2pt)
          Водитель по состоянию здоровья к управлению допущен
          #v(2pt)
          #grid(
            columns: (auto, 1fr, auto),
            column-gutter: 3pt,
            align: bottom,
            [#show_date_time(date, medical_exam_time)], [#box(width: 100%, stroke: (bottom: 0.5pt))[]], [#sub[#get_initials(medic_name)]]
          )
          #v(8pt)
          Прошел послерейсовый медицинский осмотр
          #v(2pt)
          #grid(
            columns: (auto, 1fr, auto),
            column-gutter: 3pt,
            align: bottom,
            [#show_date(date)], [#box(width: 100%, stroke: (bottom: 0.5pt))[]], [#sub[#get_initials(medic_name)]]
          )
        ],
        [
          Автомобиль технически исправен.
          #v(2pt)
          #sig_doc_line("Выезд разрешен. Механик", mechanic_name, 1fr)
          #v(2pt)
          #sig_line("Автомобиль принял: Водитель", driver, 1fr)
          #v(2pt)
          При возвращении автомобиль #empty_f(2.5cm) #sub[исправен/неисправен]
          #v(6pt)
          #sig_line("Сдал водитель", driver, 1fr)
          #v(2pt)
          #sig_doc_line("Принял механик", mechanic_name, 1fr)
        ],
        [
          Отметки организации-владельца\ автотранспорта\
          #v(4pt)
          #empty_f(100%)\
          #v(4pt)
          #empty_f(100%)\
          #v(4pt)
          #empty_f(100%)\
          #v(4pt)
          #empty_f(100%)
        ]
      )
    ]
  ]
}

#let waybill_tractor(
  driver: "",
  driving_license: "",
  driving_license_date: "",
  tractor_license: "",
  tractor_license_date: "",
  vehicle: "",
  license_plate: "",
  sts: "",
  category: "",
  date: "",
  company_name: "",
  company_address: "",
  company_inn: "",
  dispatcher_name: "",
  mechanic_name: "",
  medic_name: "",
  medical_exam_time: "07:30",
  departure_time: "",
  return_time: "",
  fuel_brand: "",
  fuel_code: "",
  fuel_issued: "",
  fuel_remain_depart: "",
  fuel_remain_return: "",
  fuel_submitted: "",
  fuel_coeff: "",
  fuel_special: "",
  fuel_engine: "",
  no_date: false,
  no_time: false,
  work_name: "",
  trailer: "",
  tractor_mode: "cargo",
  tasks: ()
) = {
  set page(paper: "a4", flipped: true, margin: 0.5cm)
  set text(font: "DejaVu Sans", size: 7.2pt)

  let field(body, w: auto) = box(width: w, stroke: (bottom: 0.5pt), inset: (bottom: 1pt), outset: (bottom: 1pt))[#body]
  let empty_f(w) = box(width: w, stroke: (bottom: 0.5pt))
  let sub(body) = text(size: 5pt)[#body]
  let h_cell(h, body) = box(height: h, width: 100%, align(center + horizon)[#body])

  // Функция для получения инициалов из полного имени
  let get_initials(name) = {
    if name == "" {
      ""
    } else {
      let parts = name.split(" ")
      if parts.len() >= 2 {
        parts.at(0) + " " + parts.at(1).at(0) + "." + if parts.len() >= 3 { parts.at(2).at(0) + "." } else { "" }
      } else { name }
    }
  }
  
  // Строка подписи: инициалы справа от прочерка (без надписи "подпись")
  let sig_line(label, name, width) = grid(
    columns: (auto, 1fr, auto),
    column-gutter: 3pt,
    align: bottom,
    [#label], [#box(width: 100%, stroke: (bottom: 0.5pt))[]], [#sub[#get_initials(name)]]
  )
  
  let sig_doc_line(label, name, width) = grid(
    columns: (auto, 1fr, auto),
    column-gutter: 3pt,
    align: bottom,
    [#label], [#box(width: 100%, stroke: (bottom: 0.5pt))[]], [#sub[#get_initials(name)]]
  )

  // Функция для отображения даты (пусто если no_date)
  let show_date(d) = {
    if no_date { "" } else { d }
  }
  
  // Функция для отображения времени (пусто если no_time)
  let show_time(t) = {
    if no_time { "" } else { t }
  }
  
  // Функция для отображения даты и времени с одним прочерком
  let show_date_time(d, t) = {
    let has_date = not no_date and d != ""
    let has_time = not no_time and t != ""
    if has_date and has_time {
      d + " " + t
    } else if has_date {
      d
    } else if has_time {
      t
    } else {
      box(width: 100%, stroke: (bottom: 0.5pt))[]
    }
  }

  // Парсим время в часы и минуты
  let parse_time(t) = {
    if t == "" or no_time {
      ("", "")
    } else if t.contains(":") {
      let parts = t.split(":")
      (parts.at(0, default: ""), parts.at(1, default: ""))
    } else if t.contains(".") {
      let parts = t.split(".")
      (parts.at(0, default: ""), parts.at(1, default: ""))
    } else {
      (t, "")
    }
  }

  // Парсим дату в день и месяц
  let parse_date(d) = {
    if d == "" or no_date {
      ("", "")
    } else if d.contains(".") {
      let parts = d.split(".")
      (parts.at(0, default: ""), parts.at(1, default: ""))
    } else if d.contains("-") {
      // ISO формат: 2026-04-15
      let parts = d.split("-")
      if parts.len() >= 3 {
        (parts.at(2, default: ""), parts.at(1, default: ""))
      } else {
        ("", "")
      }
    } else if d.contains("/") {
      let parts = d.split("/")
      (parts.at(0, default: ""), parts.at(1, default: ""))
    } else {
      (d, "")
    }
  }

  let org_line = company_name
  if company_address != "" {
    org_line = org_line + ", " + company_address
  }
  if company_inn != "" {
    org_line = org_line + ", ИНН " + company_inn
  }

  let is_cargo = tractor_mode == "cargo"

  // Тракторный путевой лист - типовая межотраслевая форма № 58
  let t1 = tasks.at(0, default: (customer: "", loading_point: "", unloading_point: "", cargo: "", trips: "", distance: "", tons: "", arrival_time: ""))
  let t2 = tasks.at(1, default: (customer: "", loading_point: "", unloading_point: "", cargo: "", trips: "", distance: "", tons: "", arrival_time: ""))

  // Парсим время выезда и возврата
  let (dep_hour, dep_min) = parse_time(departure_time)
  let (ret_hour, ret_min) = parse_time(return_time)
  // Парсим дату
  let (day, month) = parse_date(date)

  [
    #rect(width: 100%, height: 100%, stroke: 0.5pt, inset: 10pt)[
      #grid(
        columns: (1fr, 2fr, 1.2fr),
        align: (left, center, right),
        [
          #v(15pt)
        ],
        [
          #text(13pt, weight: "bold")[ПУТЕВОЙ ЛИСТ] #linebreak()
          #text(11pt, weight: "bold")[#if is_cargo [трактора с прицепом] else [трактора]] № #empty_f(2.5cm) #linebreak()
          #v(3pt)
          #let d = if not no_date and date.contains(".") { date.split(".") } else { ("  ", "  ", "    ") }
          #text(size: 10pt, weight: "bold")[#d.at(0) #if d.at(1) == "04" { "апреля" } else { d.at(1) } 20#if d.at(2).len() > 2 { d.at(2).slice(2) } else { d.at(2) } г.]
        ],
        [
          #text(size: 6pt)[
            Типовая межотраслевая форма № #if is_cargo [58-С] else [68]
            Утверждена постановлением Госкомстата России
            от 28.11.97 № 78
          ]
        ]
      )

      #v(5pt)

      #grid(
        columns: (1fr, auto),
        column-gutter: 10pt,
        [
          #grid(
            columns: (auto, 1fr),
            column-gutter: 5pt,
            row-gutter: 4pt,
            align: bottom,
            [Организация], [#field(org_line, w: 100%)
 #align(center)[#sub[наименование, адрес, номер телефона]]],
          )
          #v(4pt)
          #grid(
            columns: (auto, 1fr, auto, 2.2cm, auto, 1.5cm),
            column-gutter: 5pt,
            align: bottom,
            row-gutter: 8pt,
            [#if is_cargo [Марка трактора] else [Марка трактора]], [#field(vehicle, w: 100%)], [Гос. номер], [#field(license_plate, w: 100%)], [СТС], [#field(sts, w: 100%)],
            [Водитель / Механизатор], [#field(driver, w: 100%)], [Табельный №], [#empty_f(100%)], [], [],
          )
          #v(4pt)
          #grid(
            columns: (auto, 1.2fr, auto, 0.8fr, auto, 1fr),
            column-gutter: 5pt,
            align: bottom,
            row-gutter: 8pt,
            [Удостоверение тракториста №], [#field(tractor_license, w: 100%)], [Категория], [#field(category, w: 100%)], [Дата выдачи], [#field(tractor_license_date, w: 100%)],
            [Водительское удост. №], [#field(driving_license, w: 100%)], [], [], [Дата выдачи], [#field(driving_license_date, w: 100%)],
          )
          #if not is_cargo {
            block[
              #v(4pt)
              #grid(
                columns: (auto, 1fr, auto, 1fr),
                column-gutter: 5pt,
                align: bottom,
                [Вид работы], [#field(work_name, w: 100%)], [Прицеп/навесное], [#field(trailer, w: 100%)]
              )
            ]
          } else {
            block[
              #v(4pt)
              #grid(
                columns: (auto, 1fr, auto, 1fr),
                column-gutter: 5pt,
                align: bottom,
                [Вид работы], [#field(work_name, w: 100%)], [Прицеп], [#field(trailer, w: 100%)]
              )
            ]
          }
        ],
        [
          #grid(
            columns: (auto, auto),
            column-gutter: 5pt,
            [
              #grid(
                columns: (2.2cm, 0.8cm),
                stroke: 0.5pt,
                inset: 3pt,
                align: center,
                [], [*Код*],
                [Режим работы], [],
                [Колонна], [],
                [Бригада], []
              )
            ],
            [
              #align(right)[
                #grid(
                  columns: (1.8cm, 0.7cm, 0.7cm, 0.7cm, 0.7cm, 1.2cm, 1.6cm, 1.8cm),
                  stroke: 0.5pt,
                  inset: 2pt,
                  align: center + horizon,
                  grid.cell(rowspan: 2)[*Работа водителя и ТС*],
                  grid.cell(colspan: 4)[время по графику],
                  grid.cell(rowspan: 2)[нулевой\ проб.км],
                  grid.cell(rowspan: 2)[спидом.\ км],
                  grid.cell(rowspan: 2)[время факт.\ ч, мин.]
                  ,
                  [чис.], [мес.], [ч], [мин],
                  [1], [2], [3], [4], [5], [6], [7], [8],
                  [#h_cell(15pt, "выезд")], [#h_cell(15pt, day)], [#h_cell(15pt, month)], [#h_cell(15pt, dep_hour)], [#h_cell(15pt, dep_min)], [], [], [],
                  [#h_cell(15pt, "возврат")], [#h_cell(15pt, day)], [#h_cell(15pt, month)], [#h_cell(15pt, ret_hour)], [#h_cell(15pt, ret_min)], [], [], [],
                )
              ]
            ]
          )
          #v(4pt)
          #align(right)[
            #grid(
              columns: (0.9cm, 0.8cm, 1.1cm, 1.1cm, 1.1cm, 1cm, 1cm, 1cm, 1.2cm),
              stroke: 0.5pt,
              inset: 2pt,
              align: center + horizon,
              grid.cell(colspan: 9)[*Движение горючего*],
              grid.cell(colspan: 2)[горючее],
              grid.cell(rowspan: 2)[выдано],
              grid.cell(colspan: 2)[остаток при],
              grid.cell(rowspan: 2)[сдано],
              grid.cell(rowspan: 2)[коэфф.],
              grid.cell(colspan: 2)[Время работы],
              [марка], [код], [выезде], [возвр.], [спец.], [двиг.],
              [9], [10], [11], [12], [13], [14], [15], [16], [17],
              [#h_cell(12pt, fuel_brand)], [#h_cell(12pt, fuel_code)], [#h_cell(12pt, fuel_issued)], [#h_cell(12pt, fuel_remain_depart)], [#h_cell(12pt, fuel_remain_return)], [#h_cell(12pt, fuel_submitted)], [#h_cell(12pt, fuel_coeff)], [#h_cell(12pt, fuel_special)], [#h_cell(12pt, fuel_engine)]
            )
          ]
        ]
      )

      #v(0pt)
      #align(center)[*ЗАДАНИЕ ВОДИТЕЛЮ*]

      #if is_cargo [
        #grid(
          columns: (0.9fr, 1.5cm, 1fr, 1fr, 2.5cm, 1.2cm, 1.2cm, 1.2cm),
          stroke: 0.5pt,
          inset: 3pt,
          align: center + horizon,
          grid.cell(rowspan: 2)[В чье распоряжение\ (наименование и адрес)],
          grid.cell(rowspan: 2)[время\ приб.],
          grid.cell(colspan: 2)[адрес пункта],
          grid.cell(rowspan: 2)[наименование груза],
          grid.cell(rowspan: 2)[ездок],
          grid.cell(rowspan: 2)[расст.\ км],
          grid.cell(rowspan: 2)[тонн],
          [погрузки], [разгрузки],
          [18], [19], [20], [21], [22], [23], [24], [25],
          [#h_cell(18pt, t1.customer)], [#h_cell(18pt, t1.arrival_time)], [#h_cell(18pt, t1.loading_point)], [#h_cell(18pt, t1.unloading_point)], [#h_cell(18pt, t1.cargo)], [#h_cell(18pt, t1.trips)], [#h_cell(18pt, t1.distance)], [#h_cell(18pt, t1.tons)],
          [#h_cell(18pt, t2.customer)], [#h_cell(18pt, t2.arrival_time)], [#h_cell(18pt, t2.loading_point)], [#h_cell(18pt, t2.unloading_point)], [#h_cell(18pt, t2.cargo)], [#h_cell(18pt, t2.trips)], [#h_cell(18pt, t2.distance)], [#h_cell(18pt, t2.tons)],
          grid.cell(colspan: 7, align: right)[*Итого*], []
        )
      ] else [
        // Полевые работы - таблица с наименованием работы
        #grid(
          columns: (1.5fr, 2fr, 1fr, 1fr, 1fr, 1fr, 1.5cm),
          stroke: 0.5pt,
          inset: 3pt,
          align: center + horizon,
          [Объект (поле)],
          [Наименование работы],
          [Площадь, га],
          [Моточасы],
          [Норма выработки],
          [Факт],
          [Подпись],
          [#h_cell(18pt, t1.customer)], [#h_cell(18pt, t1.cargo)], [#h_cell(18pt, t1.loading_point)], [#h_cell(18pt, t1.arrival_time)], [#h_cell(18pt, t1.tons)], [#h_cell(18pt, t1.distance)], [#empty_f(100%)],
          [#h_cell(18pt, t2.customer)], [#h_cell(18pt, t2.cargo)], [#h_cell(18pt, t2.loading_point)], [#h_cell(18pt, t2.arrival_time)], [#h_cell(18pt, t2.tons)], [#h_cell(18pt, t2.distance)], [#empty_f(100%)],
          grid.cell(colspan: 6, align: right)[*Итого*], []
        )
      ]

      #v(6pt)
      #set text(size: 8.2pt)
      
      #grid(
        columns: (1.5fr, 1.5fr, 1fr),
        column-gutter: 15pt,
        [
          Удостоверение тракториста проверил, задание выдал,\ выдать горючее #empty_f(1.5cm) литров
          #v(2pt)
          #sig_doc_line("Диспетчер", dispatcher_name, 4cm)
          #v(2pt)
          Водитель / Механизатор по состоянию здоровья к управлению допущен
          #v(2pt)
          #grid(
            columns: (auto, 1fr, auto),
            column-gutter: 3pt,
            align: bottom,
            [#show_date_time(date, medical_exam_time)], [#box(width: 100%, stroke: (bottom: 0.5pt))[]], [#sub[#get_initials(medic_name)]]
          )
          #v(8pt)
          Прошел послерейсовый медицинский осмотр
          #v(2pt)
          #grid(
            columns: (auto, 1fr, auto),
            column-gutter: 3pt,
            align: bottom,
            [#show_date(date)], [#box(width: 100%, stroke: (bottom: 0.5pt))[]], [#sub[#get_initials(medic_name)]]
          )
        ],
        [
          #if is_cargo [
            Трактор с прицепом технически исправен.
          ] else [
            Трактор технически исправен.
          ]
          #v(2pt)
          #sig_doc_line("Выезд разрешен. Механик", mechanic_name, 1fr)
          #v(2pt)
          #sig_line("ТС принял: Водитель / Механизатор", driver, 1fr)
          #v(2pt)
          При возвращении #if is_cargo [трактор с прицепом] else [трактор] #empty_f(2.5cm) #sub[исправен/неисправен]
          #v(6pt)
          #sig_line("Сдал водитель / Механизатор", driver, 1fr)
          #v(2pt)
          #sig_doc_line("Принял механик", mechanic_name, 1fr)
        ],
        [
          Отметки организации-владельца\ автотранспорта\
          #v(4pt)
          #empty_f(100%)\
          #v(4pt)
          #empty_f(100%)\
          #v(4pt)
          #empty_f(100%)\
          #v(4pt)
          #empty_f(100%)
        ]
      )
    ]
  ]
}