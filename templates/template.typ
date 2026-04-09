#let waybill_truck(
  driver: "", driving_license: "", tractor_license: "", snils: "", 
  vehicle: "", license_plate: "", sts: "", 
  work: "", date: ""
) = {
  set page(paper: "a4", flipped: true, margin: (left: 1cm, right: 1cm, top: 1cm, bottom: 1cm))
  set text(font: "DejaVu Sans", size: 7pt)
  
  let field(body, w: 100%) = box(width: w, stroke: (bottom: 0.5pt), inset: (bottom: 2pt))[#body]
  let empty_f(..args) = box(width: args.pos().at(0, default: 100%), stroke: (bottom: 0.5pt))
  let subtext(body) = text(size: 5pt)[#body]

  [
    #grid(
      columns: (1fr, 2fr, 1fr),
      align: (left, center, right),
      [
        #v(10pt)
        #box(width: 3cm, height: 1.2cm, stroke: 0.5pt)[
          #align(center + horizon)[#text(size: 6pt)[Место для штампа\ организации]]
        ]
      ],
      [
        #text(12pt, weight: "bold")[ПУТЕВОЙ ЛИСТ] #linebreak()
        #text(10pt, weight: "bold")[грузового автомобиля] № #empty_f(2cm) #linebreak()
        #v(5pt)
        #field(date, w: 4cm)
      ],
      [
        #text(size: 6pt)[
          Типовая межотраслевая форма № 4-С\
          Утверждена постановлением Госкомстата России\
          от 28.11.97 № 78
        ]
        #v(5pt)
        #align(right)[
          #grid(
            columns: (2.5cm, 2cm),
            stroke: 0.5pt,
            align: center,
            [], [*Коды*],
            [Форма по ОКУД], [0345004],
            [по ОКПО], []
          )
        ]
      ]
    )

    #v(5pt)

    #grid(
      columns: (11cm, 1fr),
      column-gutter: 10pt,
      [
        #grid(
          columns: (auto, 1fr),
          row-gutter: 4pt,
          [Организация], [#field(" ООО 'АгроТранс', ИНН 1234567890, г. Москва ")\ #align(center)[#subtext[наименование, адрес, номер телефона]]],
        )
        #v(4pt)
        #grid(
          columns: (auto, 1fr, auto, 2cm, auto, 2cm),
          align: bottom,
          row-gutter: 6pt,
          [Марка автомобиля ], [#field(vehicle)], [ Гос. номер ], [#field(license_plate)], [ Гаражный № ], [#empty_f()],
          [Водитель ], [#field(driver)], [ Табельный № ], [#empty_f()], [], [],
        )
        #v(4pt)
        #grid(
          columns: (auto, 1fr, auto, 1fr, auto, 1fr),
          align: bottom,
          row-gutter: 6pt,
          [Удостоверение № ], [#field(driving_license)], [ Класс ], [#empty_f()], [ СНИЛС ], [#field(snils)],
          [Лиценз. карточка ], [#empty_f()], [ Регистрац. № ], [#empty_f()], [ Серия ], [#empty_f()],
          [Прицеп 1 ], [#empty_f()], [ Гос. номер ], [#empty_f()], [ Гаражный № ], [#empty_f()],
          [Прицеп 2 ], [#empty_f()], [ Гос. номер ], [#empty_f()], [ Гаражный № ], [#empty_f()],
        )
        #v(4pt)
        #grid(
          columns: (auto, 1fr),
          [Сопровожд. лица ], [#empty_f()]
        )
      ],
      [
        #grid(
          columns: (2.5cm, 1fr),
          column-gutter: 5pt,
          [
            #grid(
              columns: (2cm, 0.8cm),
              stroke: 0.5pt,
              align: center,
              [], [*Код*],
              [Режим работы], [],
              [Колонна], [],
              [Бригада], []
            )
          ],
          [
            #grid(
              columns: (1.8cm, 8pt, 8pt, 8pt, 8pt, 1.2cm, 1.5cm, 2.2cm),
              stroke: 0.5pt,
              align: center + horizon,
              grid.cell(rowspan: 2)[*Работа водителя и ТС*],
              grid.cell(colspan: 4)[время по графику],
              grid.cell(rowspan: 2)[нулевой\ проб.км],
              grid.cell(rowspan: 2)[спидометр\ км],
              grid.cell(rowspan: 2)[время факт.\ ч, мин.]
              ,
              [число], [мес], [ч], [мин],
              [1], [2], [3], [4], [5], [6], [7], [8],
              [выезд], [], [], [], [], [], [], [],
              [возврат], [], [], [], [], [], [], []
            )
          ]
        )
        #v(4pt)
        #grid(
          columns: (0.8cm, 0.8cm, 1.2cm, 1cm, 1.2cm, 1cm, 1.2cm, 1.2cm, 1.2cm),
          stroke: 0.5pt,
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
          [дизель], [], [], [], [], [], [], [], [],
          grid.cell(colspan: 2)[подпись], [запр.], [мех.], [мех.], [запр.], [дисп.], [], []
        )
      ]
    )

    #v(8pt)
    #align(center)[*ЗАДАНИЕ ВОДИТЕЛЮ*]
    #grid(
      columns: (1fr, 1.8cm, 2.2cm, 2.2cm, 2.5cm, 1.2cm, 1.2cm, 1.2cm),
      stroke: 0.5pt,
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
      [ООО "Заказчик"], [], [Склад], [Поле], [#work], [], [], [],
      [], [], [], [], [], [], [], [],
      [], [], [], [], [], [], [], [],
    )

    #v(8pt)
    #grid(
      columns: (1.5fr, 1.5fr, 1fr),
      column-gutter: 10pt,
      [
        Водительское удостоверение проверил, задание выдал,\
        выдать горючее #empty_f(1.5cm) литров\
        Диспетчер #empty_f(2cm) #empty_f(3cm)\
        #align(center)[#subtext[подпись] #h(1cm) #subtext[расшифровка подписи]]
        #v(4pt)
        Водитель по состоянию здоровья к управлению допущен\
        #empty_f(2cm) #empty_f(2cm) #empty_f(3cm)\
        #align(center)[#subtext[должность] #h(0.5cm) #subtext[подпись] #h(0.5cm) #subtext[расшифровка подписи]]
      ],
      [
        Автомобиль технически исправен. Выезд разрешен.\
        Механик #empty_f(1.5cm) #empty_f(2.5cm)\
        #align(right)[#subtext[подпись] #h(1cm) #subtext[расшифровка подписи]]
        #v(4pt)
        Автомобиль принял: Водитель #empty_f(1.5cm) #empty_f(2.5cm)\
        #align(right)[#subtext[подпись] #h(1cm) #subtext[расшифровка подписи]]
        #v(4pt)
        При возвращении автомобиль #empty_f(2cm)\
        #align(right)[#subtext[исправен / неисправен]]
        Сдал водитель #empty_f(1.5cm) #empty_f(2.5cm)\
        Принял механик #empty_f(1.5cm) #empty_f(2.5cm)\
      ],
      [
        Отметки организации-владельца\
        #v(4pt)
        #empty_f()\
        #v(4pt)
        #empty_f()\
        #v(4pt)
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
