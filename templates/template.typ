#let waybill_truck(
  driver: "", driving_license: "", tractor_license: "", snils: "", 
  vehicle: "", license_plate: "", sts: "", 
  work: "", date: ""
) = {
  set page(paper: "a4", flipped: true, margin: (left: 1cm, right: 1cm, top: 0.8cm, bottom: 0.8cm))
  set text(font: "DejaVu Sans", size: 8pt)
  
  let field(body, w: 100%) = box(width: w, stroke: (bottom: 0.5pt), inset: (bottom: 3pt))[#body]
  let empty_f(..args) = box(width: args.pos().at(0, default: 100%), stroke: (bottom: 0.5pt))
  let subtext(body) = text(size: 6pt)[#body]
  let h_cell(h, body) = box(height: h, width: 100%, align(center + horizon)[#body])

  [
    #grid(
      columns: (1fr, 2.5fr, 1.2fr),
      align: (left, center, right),
      [
        #v(5pt)
        #box(width: 3.5cm, height: 1.5cm, stroke: 0.5pt)[
          #align(center + horizon)[#text(size: 7pt)[Место для штампа\ организации]]
        ]
      ],
      [
        #text(14pt, weight: "bold")[ПУТЕВОЙ ЛИСТ] #linebreak()
        #text(11pt, weight: "bold")[грузового автомобиля] № #empty_f(2.5cm) #linebreak()
        #v(8pt)
        #field(date, w: 5cm)
      ],
      [
        #text(size: 6.5pt)[
          Типовая межотраслевая форма № 4-С\
          Утверждена постановлением Госкомстата России\
          от 28.11.97 № 78
        ]
        #v(5pt)
        #align(right)[
          #grid(
            columns: (2.8cm, 2.2cm),
            stroke: 0.5pt,
            inset: 4pt,
            align: center,
            [], [*Коды*],
            [Форма по ОКУД], [0345004],
            [по ОКПО], []
          )
        ]
      ]
    )

    #v(8pt)

    #grid(
      columns: (11.5cm, 1fr),
      column-gutter: 15pt,
      [
        #grid(
          columns: (auto, 1fr),
          row-gutter: 6pt,
          [Организация], [#field(" ООО 'АгроТранс', ИНН 1234567890, г. Москва ")\ #align(center)[#subtext[наименование, адрес, номер телефона]]],
        )
        #v(6pt)
        #grid(
          columns: (auto, 1fr, auto, 2.5cm, auto, 2cm),
          align: bottom,
          row-gutter: 10pt,
          [Марка автомобиля ], [#field(vehicle)], [ Гос. номер ], [#field(license_plate)], [ Гаражный № ], [#empty_f()],
          [Водитель ], [#field(driver)], [ Табельный № ], [#empty_f()], [], [],
        )
        #v(6pt)
        #grid(
          columns: (auto, 1fr, auto, 1.5fr, auto, 1.5fr),
          align: bottom,
          row-gutter: 10pt,
          [Удостоверение № ], [#field(driving_license)], [ Класс ], [#empty_f()], [ СНИЛС ], [#field(snils)],
          [Лиценз. карточка ], [#empty_f()], [ Регистрац. № ], [#empty_f()], [ Серия ], [#empty_f()],
          [Прицеп 1 ], [#empty_f()], [ Гос. номер ], [#empty_f()], [ Гаражный № ], [#empty_f()],
        )
        #v(6pt)
        #grid(
          columns: (auto, 1fr),
          [Сопровожд. лица ], [#empty_f()]
        )
      ],
      [
        #grid(
          columns: (auto, auto),
          column-gutter: 8pt,
          [
            #grid(
              columns: (2.2cm, 0.8cm),
              stroke: 0.5pt,
              inset: 4pt,
              align: center,
              [], [*Код*],
              [Режим работы], [],
              [Колонна], [],
              [Бригада], []
            )
          ],
          [
            #grid(
              columns: (2cm, 10pt, 10pt, 10pt, 10pt, 1.5cm, 1.8cm, 2.2cm),
              stroke: 0.5pt,
              inset: 4pt,
              align: center + horizon,
              grid.cell(rowspan: 2)[*Работа водителя и ТС*],
              grid.cell(colspan: 4)[время по графику],
              grid.cell(rowspan: 2)[нулевой\ проб.км],
              grid.cell(rowspan: 2)[спидометр\ км],
              grid.cell(rowspan: 2)[время факт.\ ч, мин.]
              ,
              [число], [мес], [ч], [мин],
              [1], [2], [3], [4], [5], [6], [7], [8],
              [#h_cell(20pt, "выезд")], [], [], [], [], [], [], [],
              [#h_cell(20pt, "возврат")], [], [], [], [], [], [], []
            )
          ]
        )
        #v(6pt)
        #grid(
          columns: (0.9cm, 0.9cm, 1.3cm, 1.2cm, 1.3cm, 1.2cm, 1.3cm, 1.3cm, 1.3cm),
          stroke: 0.5pt,
          inset: 3pt,
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
          [#h_cell(18pt, "дизель")], [], [], [], [], [], [], [], [],
          grid.cell(colspan: 2)[#h_cell(18pt, "подпись")], [запр.], [мех.], [мех.], [запр.], [дисп.], [], []
        )
      ]
    )

    #v(10pt)
    #align(center)[*ЗАДАНИЕ ВОДИТЕЛЮ*]
    #grid(
      columns: (1fr, 2cm, 2.5cm, 2.5cm, 2.5cm, 1.5cm, 1.5cm, 1.5cm),
      stroke: 0.5pt,
      inset: 6pt,
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
      [#h_cell(25pt, "ООО 'Заказчик'")], [], [Склад], [Поле], [#work], [], [], [],
      [#h_cell(25pt, "")], [], [], [], [], [], [], [],
    )

    #v(10pt)
    #grid(
      columns: (1.5fr, 1.5fr, 1fr),
      column-gutter: 15pt,
      [
        Водительское удостоверение проверил, задание выдал,\
        выдать горючее #empty_f(1.5cm) литров\
        Диспетчер #empty_f(2cm) #empty_f(3.5cm)\
        #align(center)[#subtext[подпись] #h(1cm) #subtext[расшифровка подписи]]
        #v(6pt)
        Водитель по состоянию здоровья к управлению допущен\
        #empty_f(2.5cm) #empty_f(2cm) #empty_f(3.5cm)\
        #align(center)[#subtext[должность] #h(0.5cm) #subtext[подпись] #h(0.5cm) #subtext[расшифровка подписи]]
      ],
      [
        Автомобиль технически исправен. Выезд разрешен.\
        Механик #empty_f(1.8cm) #empty_f(3cm)\
        #align(right)[#subtext[подпись] #h(1cm) #subtext[расшифровка подписи]]
        #v(6pt)
        Автомобиль принял: Водитель #empty_f(1.8cm) #empty_f(3cm)\
        #align(right)[#subtext[подпись] #h(1cm) #subtext[расшифровка подписи]]
        #v(6pt)
        При возвращении автомобиль #empty_f(2.5cm)\
        #align(right)[#subtext[исправен / неисправен]]
        Сдал водитель #empty_f(1.8cm) #empty_f(3cm)\
        Принял механик #empty_f(1.8cm) #empty_f(3cm)\
      ],
      [
        Отметки организации-владельца\
        #v(6pt)
        #empty_f()\
        #v(6pt)
        #empty_f()\
        #v(6pt)
        #empty_f()
      ]
    )
  ]
}

#let waybill_tractor(
  driver: "", driving_license: "", tractor_license: "", snils: "", 
  vehicle: "", license_plate: "", sts: "", 
  work: "", date: ""
) = {
  set page(paper: "a4", margin: 1.5cm)
  set text(font: "DejaVu Sans", size: 9pt)

  [
    #align(center, text(14pt, weight: "bold")[ПУТЕВОЙ ЛИСТ ТРАКТОРА])
    #v(0.5cm)

    #grid(
      columns: (1fr, 1fr),
      row-gutter: 1cm,
      [
        *Организация:* ООО "АгроТранс" #linebreak()
        Марка, модель: #vehicle #linebreak()
        Госномер: #license_plate #linebreak()
        СТС/ПСМ: #sts #linebreak()
        Моточасы при выезде: ..........
      ],
      [
        *Механизатор:* #driver #linebreak()
        УТМ: #tractor_license #linebreak()
        СНИЛС: #snils #linebreak()
        Дата: #date
      ]
    )
    
    #v(1cm)
    #line(length: 100%, stroke: 0.5pt)
    
    #v(0.5cm)
    #text(weight: "bold")[ЗАДАНИЕ:]
    #grid(
      columns: (2fr, 1fr, 1fr),
      stroke: 0.5pt,
      inset: 5pt,
      [*Вид работы*], [*Норма*], [*Факт*],
      [#work], [], []
    )

    #v(1cm)
    #grid(
      columns: (1fr, 1fr),
      [Подпись агронома: #box(width: 3cm, stroke: (bottom: 0.5pt))],
      [Подпись механизатора: #box(width: 3cm, stroke: (bottom: 0.5pt))]
    )
  ]
}
