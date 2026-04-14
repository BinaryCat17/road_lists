#let waybill_truck(
  driver: "", driving_license: "", tractor_license: "", snils: "",
  vehicle: "", license_plate: "", sts: "", category: "",
  date: "",
  company_name: "", company_address: "", company_inn: "",
  dispatcher_name: "", mechanic_name: "", medic_name: "",
  tasks: ()
) = {
  set page(paper: "a4", flipped: true, margin: 0.5cm)
  set text(font: "DejaVu Sans", size: 7.2pt)

  let field(body, w: auto) = box(width: w, stroke: (bottom: 0.5pt), inset: (bottom: 1pt), outset: (bottom: 1pt))[#body]
  let empty_f(w) = box(width: w, stroke: (bottom: 0.5pt))
  let sub(body) = text(size: 5pt)[#body]
  let h_cell(h, body) = box(height: h, width: 100%, align(center + horizon)[#body])

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
          #let d = if date.contains(".") { date.split(".") } else { ("  ", "  ", "    ") }
          #text(size: 10pt, weight: "bold")[#d.at(0) #if d.at(1) == "04" { "апреля" } else { d.at(1) } 20#if d.at(2).len() > 2 { d.at(2).slice(2) } else { d.at(2) } г.]
        ],
        [
          #text(size: 6pt)[
            Типовая межотраслевая форма № 4-С\
            Утверждена постановлением Госкомстата России\
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
            [Организация], [#field(org_line, w: 100%)\ #align(center)[#sub[наименование, адрес, номер телефона]]],
          )
          #v(4pt)
          #grid(
            columns: (auto, 1fr, auto, 2.2cm, auto, 1.5cm),
            column-gutter: 5pt,
            align: bottom,
            row-gutter: 8pt,
            [Марка автомобиля], [#field(vehicle, w: 100%)], [Гос. номер], [#field(license_plate, w: 100%)], [Гаражный №], [#empty_f(100%)],
            [Водитель], [#field(driver, w: 100%)], [Табельный №], [#empty_f(100%)], [], [],
          )
          #v(4pt)
          #grid(
            columns: (auto, 1.5fr, auto, 1fr, auto, 1fr),
            column-gutter: 5pt,
            align: bottom,
            row-gutter: 8pt,
            [Удостоверение №], [#field(driving_license, w: 100%)], [Категория], [#field(category, w: 100%)], [СНИЛС], [#field(snils, w: 100%)],
            [Лиценз. карточка], [#empty_f(100%)], [Рег. №], [#empty_f(100%)], [Серия], [#empty_f(100%)],
            [Прицеп 1], [#empty_f(100%)], [Гос. номер], [#empty_f(100%)], [Гаражный №], [#empty_f(100%)],
            [Прицеп 2], [#empty_f(100%)], [Гос. номер], [#empty_f(100%)], [Гаражный №], [#empty_f(100%)],
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
                  [#h_cell(15pt, "выезд")], [], [], [], [], [], [], [],
                  [#h_cell(15pt, "возврат")], [], [], [], [], [], [], [],
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
              [#h_cell(12pt, "дизель")], [], [], [], [], [], [], [], []
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
      #let sig_row(label, w) = grid(
        columns: (auto, w),
        column-gutter: 5pt,
        row-gutter: 0pt,
        align: bottom,
        [#label], [#box(width: 100%, stroke: (bottom: 0.5pt), inset: (bottom: 1pt), outset: (bottom: 1pt))[#hide[M]]],
        [], [#align(center)[#sub[подпись] #h(1cm) #sub[расшифровка подписи]]]
      )
      #let sig_doc_row(label, w) = grid(
        columns: (auto, w),
        column-gutter: 5pt,
        row-gutter: 0pt,
        align: bottom,
        [#label], [#box(width: 100%, stroke: (bottom: 0.5pt), inset: (bottom: 1pt), outset: (bottom: 1pt))[#hide[M]]],
        [], [#align(center)[#sub[должность] #h(0.5cm) #sub[подпись] #h(0.5cm) #sub[расшифровка подписи]]]
      )
      #let sig_med_row(left_text) = grid(
        columns: (auto, 1fr),
        column-gutter: 5pt,
        row-gutter: 0pt,
        align: bottom,
        [#left_text], [#box(width: 100%, stroke: (bottom: 0.5pt), inset: (bottom: 1pt), outset: (bottom: 1pt))[#hide[M]]],
        [], [#align(center)[#sub[должность] #h(0.5cm) #sub[подпись] #h(0.5cm) #sub[расшифровка подписи]]]
      )
      #grid(
        columns: (1.5fr, 1.5fr, 1fr),
        column-gutter: 15pt,
        [
          Водительское удостоверение проверил, задание выдал,\
          выдать горючее #empty_f(1.5cm) литров
          #v(2pt)
          #sig_doc_row("Диспетчер " + dispatcher_name, 4cm)
          #v(2pt)
          Водитель по состоянию здоровья к управлению допущен
          #v(2pt)
          #sig_med_row([#date 07:30])
          #v(8pt)
          Прошел послерейсовый медицинский осмотр
          #v(2pt)
          #sig_med_row([#empty_f(0.5cm) #empty_f(1.2cm) 20#empty_f(0.5cm) г. #empty_f(0.5cm):#empty_f(0.5cm)])
        ],
        [
          Автомобиль технически исправен.
          #v(2pt)
          #sig_doc_row("Выезд разрешен. Механик " + mechanic_name, 1fr)
          #v(2pt)
          #sig_row("Автомобиль принял: Водитель", 1fr)
          #v(2pt)
          При возвращении автомобиль #empty_f(2.5cm) #sub[исправен/неисправен]
          #v(6pt)
          #sig_row("Сдал водитель", 1fr)
          #v(2pt)
          #sig_doc_row("Принял механик " + mechanic_name, 1fr)
        ],
        [
          Отметки организации-владельца\
          автотранспорта\
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
  driver: "", driving_license: "", tractor_license: "", snils: "",
  vehicle: "", license_plate: "", sts: "", category: "",
  date: "",
  company_name: "", company_address: "", company_inn: "",
  dispatcher_name: "", mechanic_name: "", medic_name: "",
  tasks: (),
  tractor_mode: "cargo"
) = {
  set page(paper: "a4", flipped: true, margin: 0.5cm)
  set text(font: "DejaVu Sans", size: 7.2pt)

  let field(body, w: auto) = box(width: w, stroke: (bottom: 0.5pt), inset: (bottom: 1pt), outset: (bottom: 1pt))[#body]
  let empty_f(w) = box(width: w, stroke: (bottom: 0.5pt))
  let sub(body) = text(size: 5pt)[#body]
  let h_cell(h, body) = box(height: h, width: 100%, align(center + horizon)[#body])

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
          #text(11pt, weight: "bold")[трактора] № #empty_f(2.5cm) #linebreak()
          #v(3pt)
          #let d = if date.contains(".") { date.split(".") } else { ("  ", "  ", "    ") }
          #text(size: 10pt, weight: "bold")[#d.at(0) #if d.at(1) == "04" { "апреля" } else { d.at(1) } 20#if d.at(2).len() > 2 { d.at(2).slice(2) } else { d.at(2) } г.]
        ],
        [
          #text(size: 6pt)[
            Форма 412-АПК\
            (рекомендована Минсельхозом России)
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
            [Организация], [#field(org_line, w: 100%)\ #align(center)[#sub[наименование, адрес, номер телефона]]],
          )
          #v(4pt)
          #grid(
            columns: (auto, 1fr, auto, 2.2cm, auto, 1.5cm),
            column-gutter: 5pt,
            align: bottom,
            row-gutter: 8pt,
            [Марка трактора], [#field(vehicle, w: 100%)], [Гос. номер], [#field(license_plate, w: 100%)], [Инвент. №], [#empty_f(100%)],
            [Тракторист], [#field(driver, w: 100%)], [Табельный №], [#empty_f(100%)], [], [],
          )
          #v(4pt)
          #grid(
            columns: (auto, 1.5fr, auto, 1fr, auto, 1fr),
            column-gutter: 5pt,
            align: bottom,
            row-gutter: 8pt,
            [Удостоверение тракториста №], [#field(tractor_license, w: 100%)], [Категория], [#field(category, w: 100%)], [СНИЛС], [#field(snils, w: 100%)],
            [СТС / ПСМ], [#field(sts, w: 100%)], [Рег. №], [#empty_f(100%)], [Серия], [#empty_f(100%)],
            [Прицеп 1], [#empty_f(100%)], [Гос. номер], [#empty_f(100%)], [Гаражный №], [#empty_f(100%)],
            [Прицеп 2], [#empty_f(100%)], [Гос. номер], [#empty_f(100%)], [Гаражный №], [#empty_f(100%)],
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
                  grid.cell(rowspan: 2)[*Работа тракториста*],
                  grid.cell(colspan: 4)[время по графику],
                  grid.cell(rowspan: 2)[пробег,\ км],
                  grid.cell(rowspan: 2)[мото-часы],
                  grid.cell(rowspan: 2)[время факт.\ ч, мин.]
                  ,
                  [чис.], [мес.], [ч], [мин],
                  [1], [2], [3], [4], [5], [6], [7], [8],
                  [#h_cell(15pt, "выезд")], [], [], [], [], [], [], [],
                  [#h_cell(15pt, "возврат")], [], [], [], [], [], [], [],
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
              [#h_cell(12pt, "дизель")], [], [], [], [], [], [], [], []
            )
          ]
        ]
      )

      #v(0pt)
      #align(center)[*ЗАДАНИЕ ТРАКТОРИСТУ*]
      #v(2pt)
      #if tractor_mode == "cargo" [
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
      ] else [
        #grid(
          columns: (2fr, 1fr, 1fr, 1fr, 1fr),
          stroke: 0.5pt,
          inset: 3pt,
          align: center + horizon,
          [Объект (поле, участок)], [Площадь, га], [Норма выработки], [Факт], [Моточасы],
          [#h_cell(18pt, t1.loading_point)], [#h_cell(18pt, t1.distance)], [#h_cell(18pt, t1.tons)], [#h_cell(18pt, t1.trips)], [#h_cell(18pt, t1.arrival_time)],
          [#h_cell(18pt, t2.loading_point)], [#h_cell(18pt, t2.distance)], [#h_cell(18pt, t2.tons)], [#h_cell(18pt, t2.trips)], [#h_cell(18pt, t2.arrival_time)],
          [#h_cell(18pt, t3.loading_point)], [#h_cell(18pt, t3.distance)], [#h_cell(18pt, t3.tons)], [#h_cell(18pt, t3.trips)], [#h_cell(18pt, t3.arrival_time)],
          grid.cell(colspan: 4, align: right)[*Итого*], []
        )
      ]

      #v(6pt)
      #set text(size: 8.2pt)
      #let sig_row(label, w) = grid(
        columns: (auto, w),
        column-gutter: 5pt,
        row-gutter: 0pt,
        align: bottom,
        [#label], [#box(width: 100%, stroke: (bottom: 0.5pt), inset: (bottom: 1pt), outset: (bottom: 1pt))[#hide[M]]],
        [], [#align(center)[#sub[подпись] #h(1cm) #sub[расшифровка подписи]]]
      )
      #let sig_doc_row(label, w) = grid(
        columns: (auto, w),
        column-gutter: 5pt,
        row-gutter: 0pt,
        align: bottom,
        [#label], [#box(width: 100%, stroke: (bottom: 0.5pt), inset: (bottom: 1pt), outset: (bottom: 1pt))[#hide[M]]],
        [], [#align(center)[#sub[должность] #h(0.5cm) #sub[подпись] #h(0.5cm) #sub[расшифровка подписи]]]
      )
      #let sig_med_row(left_text) = grid(
        columns: (auto, 1fr),
        column-gutter: 5pt,
        row-gutter: 0pt,
        align: bottom,
        [#left_text], [#box(width: 100%, stroke: (bottom: 0.5pt), inset: (bottom: 1pt), outset: (bottom: 1pt))[#hide[M]]],
        [], [#align(center)[#sub[должность] #h(0.5cm) #sub[подпись] #h(0.5cm) #sub[расшифровка подписи]]]
      )
      #grid(
        columns: (1.5fr, 1.5fr, 1fr),
        column-gutter: 15pt,
        [
          Удостоверение тракториста проверил, задание выдал,\
          выдать горючее #empty_f(1.5cm) литров
          #v(2pt)
          #sig_doc_row("Диспетчер " + dispatcher_name, 4cm)
          #v(2pt)
          Тракторист по состоянию здоровья к управлению допущен
          #v(2pt)
          #sig_med_row([#date 07:30])
          #v(8pt)
          Прошел послерейсовый медицинский осмотр
          #v(2pt)
          #sig_med_row([#empty_f(0.5cm) #empty_f(1.2cm) 20#empty_f(0.5cm) г. #empty_f(0.5cm):#empty_f(0.5cm)])
        ],
        [
          Трактор технически исправен.
          #v(2pt)
          #sig_doc_row("Выезд разрешен. Механик " + mechanic_name, 1fr)
          #v(2pt)
          #sig_row("Трактор принял: Тракторист", 1fr)
          #v(2pt)
          При возвращении трактор #empty_f(2.5cm) #sub[исправен/неисправен]
          #v(6pt)
          #sig_row("Сдал тракторист", 1fr)
          #v(2pt)
          #sig_doc_row("Принял механик " + mechanic_name, 1fr)
        ],
        [
          Отметки организации-владельца\
          трактора\
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
