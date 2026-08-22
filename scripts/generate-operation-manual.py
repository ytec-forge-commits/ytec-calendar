"""Koyomadoの配布用PDF操作説明書を生成する。"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER, TA_LEFT
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import mm
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (
    BaseDocTemplate,
    Frame,
    Image,
    KeepTogether,
    PageBreak,
    PageTemplate,
    Paragraph,
    Spacer,
    Table,
    TableStyle,
)


ROOT = Path(__file__).resolve().parents[1]
ASSET_DIR = ROOT / "docs" / "manual-assets"
VERSION = json.loads((ROOT / "package.json").read_text(encoding="utf-8"))["version"]
RELEASE_DATE = "2026年8月22日"
OFFICIAL_URL = "https://ytec.cloudfree.jp/ytb/koyomado/"
SOURCE_URL = "https://github.com/ytec-commits/ytec-calendar"
GOOGLE_CONSOLE_URL = "https://console.cloud.google.com/"
GOOGLE_CREDENTIALS_URL = "https://developers.google.com/workspace/guides/create-credentials#desktop-app"
GOOGLE_USER_DATA_URL = "https://developers.google.com/terms/api-services-user-data-policy"
TOTAL_PAGES = 18

PAGE_W, PAGE_H = A4
MARGIN_X = 17 * mm
MARGIN_TOP = 16 * mm
MARGIN_BOTTOM = 16 * mm
CONTENT_W = PAGE_W - 2 * MARGIN_X

INK = colors.HexColor("#343041")
MUTED = colors.HexColor("#716b7c")
PURPLE = colors.HexColor("#806f9f")
PURPLE_DARK = colors.HexColor("#5f5278")
PURPLE_PALE = colors.HexColor("#f1edf7")
GREEN = colors.HexColor("#78a88f")
GREEN_PALE = colors.HexColor("#eaf3ee")
SKY_PALE = colors.HexColor("#eaf2f7")
SAND_PALE = colors.HexColor("#f7f1e6")
ROSE_PALE = colors.HexColor("#f8ecee")
LINE = colors.HexColor("#ded9e6")
WHITE = colors.white


def register_fonts() -> None:
    regular = ROOT / "src" / "assets" / "fonts" / "LINESeedJP-Regular.ttf"
    bold = ROOT / "src" / "assets" / "fonts" / "LINESeedJP-Bold.ttf"
    if not regular.exists() or not bold.exists():
        raise FileNotFoundError("同梱したLINE Seed JPフォントが見つかりません。")
    pdfmetrics.registerFont(TTFont("KoyomadoRegular", str(regular)))
    pdfmetrics.registerFont(TTFont("KoyomadoBold", str(bold)))


def build_styles() -> dict[str, ParagraphStyle]:
    samples = getSampleStyleSheet()
    base = dict(
        fontName="KoyomadoRegular",
        textColor=INK,
        wordWrap="CJK",
        splitLongWords=True,
    )
    return {
        "cover_kicker": ParagraphStyle(
            "CoverKicker", parent=samples["Normal"], fontName="KoyomadoBold",
            fontSize=10, leading=14, textColor=PURPLE, alignment=TA_CENTER,
            spaceAfter=3 * mm, wordWrap="CJK",
        ),
        "cover_title": ParagraphStyle(
            "CoverTitle", parent=samples["Title"], fontName="KoyomadoBold",
            fontSize=30, leading=36, textColor=INK, alignment=TA_CENTER,
            spaceAfter=2 * mm, wordWrap="CJK",
        ),
        "cover_subtitle": ParagraphStyle(
            "CoverSubtitle", parent=samples["Normal"], fontName="KoyomadoRegular",
            fontSize=11.5, leading=18, textColor=MUTED, alignment=TA_CENTER,
            spaceAfter=7 * mm, wordWrap="CJK",
        ),
        "page_kicker": ParagraphStyle(
            "PageKicker", parent=samples["Normal"], fontName="KoyomadoBold",
            fontSize=8, leading=11, textColor=PURPLE, spaceAfter=1.5 * mm,
            wordWrap="CJK",
        ),
        "h1": ParagraphStyle(
            "H1", parent=samples["Heading1"], fontName="KoyomadoBold",
            fontSize=21, leading=27, textColor=INK, spaceAfter=4 * mm,
            wordWrap="CJK",
        ),
        "h2": ParagraphStyle(
            "H2", parent=samples["Heading2"], fontName="KoyomadoBold",
            fontSize=13.5, leading=19, textColor=PURPLE_DARK,
            spaceBefore=3 * mm, spaceAfter=2 * mm, wordWrap="CJK",
        ),
        "h3": ParagraphStyle(
            "H3", parent=samples["Heading3"], fontName="KoyomadoBold",
            fontSize=10.5, leading=15, textColor=INK, spaceAfter=1.2 * mm,
            wordWrap="CJK",
        ),
        "body": ParagraphStyle(
            "Body", parent=samples["BodyText"], fontSize=9.4, leading=15,
            spaceAfter=2.1 * mm, **base,
        ),
        "body_small": ParagraphStyle(
            "BodySmall", parent=samples["BodyText"], fontSize=8.3, leading=13,
            spaceAfter=1.5 * mm, **base,
        ),
        "body_tiny": ParagraphStyle(
            "BodyTiny", parent=samples["BodyText"], fontSize=7.3, leading=10.5,
            spaceAfter=0, **base,
        ),
        "muted": ParagraphStyle(
            "Muted", parent=samples["BodyText"], fontName="KoyomadoRegular",
            fontSize=8.2, leading=13, textColor=MUTED, wordWrap="CJK",
        ),
        "step_no": ParagraphStyle(
            "StepNo", parent=samples["Normal"], fontName="KoyomadoBold",
            fontSize=16, leading=20, textColor=WHITE, alignment=TA_CENTER,
        ),
        "card_title": ParagraphStyle(
            "CardTitle", parent=samples["Normal"], fontName="KoyomadoBold",
            fontSize=10, leading=14, textColor=INK, spaceAfter=1 * mm,
            wordWrap="CJK",
        ),
        "card_body": ParagraphStyle(
            "CardBody", parent=samples["Normal"], fontName="KoyomadoRegular",
            fontSize=8.4, leading=13, textColor=INK, wordWrap="CJK",
        ),
        "code": ParagraphStyle(
            "Code", parent=samples["Code"], fontName="KoyomadoRegular",
            fontSize=7.8, leading=12, textColor=PURPLE_DARK, backColor=PURPLE_PALE,
            borderPadding=2 * mm, wordWrap="CJK",
        ),
        "footer": ParagraphStyle(
            "Footer", parent=samples["Normal"], fontName="KoyomadoRegular",
            fontSize=7.2, leading=9, textColor=MUTED, alignment=TA_CENTER,
        ),
        "link": ParagraphStyle(
            "Link", parent=samples["Normal"], fontName="KoyomadoRegular",
            fontSize=8.5, leading=13, textColor=PURPLE_DARK, alignment=TA_CENTER,
            wordWrap="CJK",
        ),
    }


def p(text: str, style: ParagraphStyle) -> Paragraph:
    return Paragraph(text, style)


def page_title(story: list, styles: dict[str, ParagraphStyle], kicker: str, title: str, lead: str | None = None) -> None:
    story.append(p(kicker.upper(), styles["page_kicker"]))
    story.append(p(title, styles["h1"]))
    if lead:
        story.append(p(lead, styles["body"]))


def screenshot(filename: str, width: float = 164 * mm) -> Image:
    path = ASSET_DIR / filename
    if not path.exists():
        raise FileNotFoundError(f"説明書用画像が見つかりません: {path}")
    image = Image(str(path))
    aspect_ratio = image.imageHeight / image.imageWidth
    image.drawWidth = width
    image.drawHeight = width * aspect_ratio
    image.hAlign = "CENTER"
    return image


def card(title: str, body: str, styles: dict[str, ParagraphStyle], background=PURPLE_PALE) -> Table:
    inner = [p(title, styles["card_title"]), p(body, styles["card_body"])]
    table = Table([[inner]], colWidths=[CONTENT_W], hAlign="LEFT")
    table.setStyle(TableStyle([
        ("BACKGROUND", (0, 0), (-1, -1), background),
        ("BOX", (0, 0), (-1, -1), 0.7, LINE),
        ("LEFTPADDING", (0, 0), (-1, -1), 4 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 4 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 3 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 3 * mm),
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
    ]))
    return table


def two_cards(items: list[tuple[str, str]], styles: dict[str, ParagraphStyle], backgrounds=None) -> Table:
    backgrounds = backgrounds or [WHITE] * len(items)
    cells = []
    for title, body in items:
        cells.append([p(title, styles["card_title"]), p(body, styles["card_body"])])
    cols = len(cells)
    gap = 4 * mm
    col_w = (CONTENT_W - gap * (cols - 1)) / cols
    row = []
    for index, cell in enumerate(cells):
        row.append(cell)
        if index < cols - 1:
            row.append("")
    widths = []
    for index in range(cols):
        widths.append(col_w)
        if index < cols - 1:
            widths.append(gap)
    table = Table([row], colWidths=widths, hAlign="LEFT")
    commands = [
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("LEFTPADDING", (0, 0), (-1, -1), 3.2 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 3.2 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 3 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 3 * mm),
    ]
    for index, bg in enumerate(backgrounds):
        cell_index = index * 2
        commands.extend([
            ("BACKGROUND", (cell_index, 0), (cell_index, 0), bg),
            ("BOX", (cell_index, 0), (cell_index, 0), 0.7, LINE),
        ])
    for index in range(cols - 1):
        spacer_index = index * 2 + 1
        commands.extend([
            ("LEFTPADDING", (spacer_index, 0), (spacer_index, 0), 0),
            ("RIGHTPADDING", (spacer_index, 0), (spacer_index, 0), 0),
        ])
    table.setStyle(TableStyle(commands))
    return table


def step(number: int, title: str, body: str, styles: dict[str, ParagraphStyle]) -> Table:
    circle = Table([[p(str(number), styles["step_no"])]], colWidths=[12 * mm], rowHeights=[12 * mm])
    circle.setStyle(TableStyle([
        ("BACKGROUND", (0, 0), (-1, -1), PURPLE),
        ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
        ("LEFTPADDING", (0, 0), (-1, -1), 0),
        ("RIGHTPADDING", (0, 0), (-1, -1), 0),
        ("TOPPADDING", (0, 0), (-1, -1), 0),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 0),
    ]))
    text = [p(title, styles["card_title"]), p(body, styles["card_body"])]
    table = Table([[circle, text]], colWidths=[16 * mm, CONTENT_W - 16 * mm], hAlign="LEFT")
    table.setStyle(TableStyle([
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("LEFTPADDING", (0, 0), (-1, -1), 0),
        ("RIGHTPADDING", (0, 0), (-1, -1), 0),
        ("TOPPADDING", (0, 0), (-1, -1), 1.7 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 2.2 * mm),
        ("LINEBELOW", (0, 0), (-1, -1), 0.5, LINE),
    ]))
    return table


def bullet(text: str, styles: dict[str, ParagraphStyle]) -> Paragraph:
    style = ParagraphStyle(
        "BulletInline", parent=styles["body"], leftIndent=5 * mm,
        firstLineIndent=-4 * mm, spaceAfter=1.5 * mm,
    )
    return Paragraph(f"・{text}", style)


def data_table(rows: list[tuple[str, str]], styles: dict[str, ParagraphStyle], first_col=44 * mm) -> Table:
    body = [[p(a, styles["body_small"]), p(b, styles["body_small"])] for a, b in rows]
    table = Table(body, colWidths=[first_col, CONTENT_W - first_col], repeatRows=0, hAlign="LEFT")
    commands = [
        ("BOX", (0, 0), (-1, -1), 0.7, LINE),
        ("INNERGRID", (0, 0), (-1, -1), 0.4, LINE),
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("LEFTPADDING", (0, 0), (-1, -1), 3 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 3 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 2.2 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 2.2 * mm),
    ]
    for index in range(len(body)):
        commands.append(("BACKGROUND", (0, index), (0, index), PURPLE_PALE if index % 2 == 0 else SKY_PALE))
    table.setStyle(TableStyle(commands))
    return table


def decorate_page(canvas, doc) -> None:
    canvas.saveState()
    if doc.page > 1:
        canvas.setStrokeColor(LINE)
        canvas.setLineWidth(0.5)
        canvas.line(MARGIN_X, 12 * mm, PAGE_W - MARGIN_X, 12 * mm)
        canvas.setFont("KoyomadoRegular", 7.2)
        canvas.setFillColor(MUTED)
        canvas.drawString(MARGIN_X, 7.8 * mm, f"Koyomado 操作説明書  v{VERSION}")
        canvas.setFont("Helvetica", 7.2)
        canvas.drawRightString(PAGE_W - MARGIN_X, 7.8 * mm, f"{doc.page} / {TOTAL_PAGES}")
    canvas.restoreState()


def build_story(styles: dict[str, ParagraphStyle]) -> list:
    story: list = []

    # 1: cover
    logo = Image(str(ROOT / "src" / "assets" / "koyomado-logo.png"), 22 * mm, 22 * mm)
    logo.hAlign = "CENTER"
    story.extend([
        Spacer(1, 4 * mm),
        logo,
        Spacer(1, 5 * mm),
        p("WINDOWS PORTABLE CALENDAR", styles["cover_kicker"]),
        p("Koyomado 操作説明書", styles["cover_title"]),
        p("予定を、いつでも目に入る場所へ。<br/>デスクトップにそっと置いて使える、シンプルな月カレンダーです。", styles["cover_subtitle"]),
        screenshot("calendar-v1.png", 156 * mm),
        Spacer(1, 6 * mm),
        two_cards([
            ("対応環境", "Windows 10 / 11（64bit）<br/>インストール不要"),
            ("この説明書", f"Koyomado v{VERSION}<br/>{RELEASE_DATE}・署名前ベータ版"),
            ("保存と通信", "予定はアプリ横へ保存<br/>Google連携は任意・初期OFF"),
        ], styles, [PURPLE_PALE, GREEN_PALE, SKY_PALE]),
        Spacer(1, 5 * mm),
        p(f'<link href="{OFFICIAL_URL}" color="#5f5278">公式ページ: {OFFICIAL_URL}</link>', styles["link"]),
        PageBreak(),
    ])

    # 2: quick start
    page_title(story, styles, "GETTING STARTED", "はじめに - 4ステップで使い始める", "Koyomadoはインストーラーを使わないポータブルアプリです。ZIPを展開したフォルダーが、そのままアプリ本体と保存場所になります。")
    story.extend([
        step(1, "ZIPをすべて展開", "ダウンロードしたZIPを右クリックし、Windowsの「すべて展開」を選びます。ZIPの中から直接起動せず、先に展開してください。", styles),
        step(2, "置き場所を決める", "展開したKoyomadoフォルダーを、ドキュメント、USBメモリ、Google Driveなど今後使う場所へ移します。自動起動をONにした後の移動は避けてください。", styles),
        step(3, "koyomado.exeを起動", "初回起動時、Windowsの警告が出る場合があります。公式ページから入手したファイルであることとSHA-256を確認し、不安がある場合は実行しないでください。", styles),
        step(4, "位置と起動方法を整える", "画面を好きな位置とサイズに調整します。右上の歯車から、必要な場合だけ「Windows起動時に自動起動」をONにします。", styles),
        Spacer(1, 4 * mm),
        card("標準はタスクバーだけに表示", "初期設定では通常のWindowsアプリと同じく、最小化するとタスクバーへ残り、右上の×で終了します。歯車から「タスクトレイのみ」または「両方」へ変更できます。", styles, GREEN_PALE),
        Spacer(1, 4 * mm),
        p("Windowsの警告について", styles["h2"]),
        p(f"v{VERSION}はコード署名前のベータ版です。SmartScreenなどの警告は、危険と確定したという意味ではなく、発行元を署名で確認できない場合にも表示されます。公式ページ掲載のSHA-256とダウンロードしたZIPの値を照合し、入手元を確認してください。", styles["body"]),
        p("PowerShellで確認する場合", styles["h3"]),
        p(f"Get-FileHash .\\koyomado-v{VERSION}-windows-portable.zip -Algorithm SHA256", styles["code"]),
        PageBreak(),
    ])

    # 3: screen overview
    page_title(story, styles, "SCREEN", "画面の見かた", "中央が月カレンダー、左が今日と直近7日間の予定を示すサイドバーです。")
    story.extend([
        screenshot("calendar-v1.png", 155 * mm),
        Spacer(1, 4 * mm),
        two_cards([
            ("1  月を移動", "上部の左右矢印で前月・翌月へ移動。「今日」は、押した時点の現在日へ戻ります。"),
            ("2  予定を追加", "予定がない日付を左クリックするか、右上・日付内・左側の追加ボタンから登録できます。"),
        ], styles, [PURPLE_PALE, GREEN_PALE]),
        Spacer(1, 3 * mm),
        two_cards([
            ("3  日の予定を確認", "予定のある日付を左クリックすると、その日の予定一覧がポップアップ表示されます。"),
            ("4  表示を整える", "上部のボタンでサイドバーを開閉。歯車で背景、表示先、自動起動、Google連携を設定します。"),
        ], styles, [SKY_PALE, SAND_PALE]),
        Spacer(1, 3 * mm),
        p("土曜は青系、日曜と祝日は赤系で表示します。日本の祝日は名前も日付内に表示されます。祝日データはオフラインで、内蔵範囲は1970年から2050年です。", styles["muted"]),
        PageBreak(),
    ])

    # 4: create/edit/delete
    page_title(story, styles, "SCHEDULE", "予定を追加・編集・削除する", "開始日と終了日を持つ予定を登録できます。必要に応じて時刻、場所、メモ、色を加えてください。")
    image = screenshot("period-editor-v1.png", 96 * mm)
    details = [
        p("入力できる内容", styles["h2"]),
        bullet("<b>予定名</b>: 必須、80文字まで", styles),
        bullet("<b>開始日・終了日</b>: 同日または複数日を指定", styles),
        bullet("<b>終日</b>: 休み、出張、記念日など時刻が不要な予定", styles),
        bullet("<b>開始・終了時刻</b>: 終日をOFFにすると表示", styles),
        bullet("<b>繰り返し</b>: 毎日、毎週、毎月、毎年", styles),
        bullet("<b>場所</b>: 任意、100文字まで", styles),
        bullet("<b>メモ</b>: 任意、1000文字まで", styles),
        bullet("<b>予定の色</b>: 6色から選択", styles),
        Spacer(1, 2 * mm),
        p("開始時刻を変えると、終了は1時間後へ自動設定されます。その後に終了日・終了時刻を手動で変更できます。", styles["body_small"]),
    ]
    side = Table([[image, details]], colWidths=[100 * mm, CONTENT_W - 100 * mm], hAlign="LEFT")
    side.setStyle(TableStyle([
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("LEFTPADDING", (0, 0), (-1, -1), 0),
        ("RIGHTPADDING", (0, 0), (0, 0), 4 * mm),
        ("RIGHTPADDING", (1, 0), (1, 0), 0),
        ("TOPPADDING", (0, 0), (-1, -1), 0),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 0),
    ]))
    story.extend([
        side,
        Spacer(1, 4 * mm),
        two_cards([
            ("編集する", "カレンダー上の予定を左クリックするか、日の予定一覧から選ぶと編集画面が開きます。「変更を保存」で反映します。"),
            ("削除する", "編集画面左下、または予定の右クリックメニューから「削除」を選び、確認画面で確定します。削除済み記録は保存データ内に残ります。"),
        ], styles, [SKY_PALE, ROSE_PALE]),
        Spacer(1, 3 * mm),
        card("同じ日に予定が多い場合", "月表示に収まらない予定は「ほか○件」とまとめて表示します。日付を選ぶと一覧ですべて確認でき、5件以上登録した場合もここから編集できます。", styles, GREEN_PALE),
        PageBreak(),
    ])

    # 5: multi-day events
    page_title(story, styles, "MULTI-DAY", "連休・出張・日をまたぐ予定", "9月1日から3日までの休みなどは、1件の期間予定として登録できます。")
    story.extend([
        screenshot("multi-day-v1.png", 152 * mm),
        Spacer(1, 4 * mm),
        two_cards([
            ("終日の複数日予定", "開始日を9月1日、終了日を9月3日、終日をONにすると、1日・2日・3日の各日に同じ予定を表示します。"),
            ("時刻付きの日またぎ", "終日をOFFにし、開始を9月1日23:30、終了を9月2日0:30のように指定します。2日以上先も選べます。"),
        ], styles, [GREEN_PALE, SKY_PALE]),
        Spacer(1, 3 * mm),
        data_table([
            ("開始時刻を変更", "終了日時を開始の1時間後へ自動設定。23:30なら翌日0:30"),
            ("終了日時を変更", "自動設定後も、終了日と終了時刻を自由に上書き可能"),
            ("カレンダー表示", "期間中の各日に表示。どの日から開いても同じ開始・終了日時を編集"),
            ("直近7日間", "同じ期間予定を日数分重複させず、1件として表示"),
        ], styles, 46 * mm),
        Spacer(1, 3 * mm),
        card("Googleカレンダーとの期間同期", "複数日の終日予定と日をまたぐ時刻付き予定も、開始・終了日時を保って双方向同期します。終日予定の終了日が1日ずれないよう、Google側の排他的終了日をKoyomado内で変換します。", styles, PURPLE_PALE),
        PageBreak(),
    ])

    # 6: copy and drag
    page_title(story, styles, "COPY AND MOVE", "予定をコピー・移動する", "繰り返し入力する内容は右クリック、日付だけ変えたいときはドラッグ操作が便利です。")
    story.extend([
        two_cards([
            ("右クリックでコピー", "1. 予定を右クリック<br/>2. 「内容をコピー」<br/>3. 貼り付け先の日付を右クリック<br/>4. 「ここに貼り付け」"),
            ("ドラッグで移動", "予定をつかみ、別の日へドラッグして離します。元の日から予定が移動します。"),
            ("Ctrl + ドラッグでコピー", "Ctrlキーを押したまま予定を別の日へドラッグ。元を残し、移動先へ複製します。"),
        ], styles, [PURPLE_PALE, SKY_PALE, GREEN_PALE]),
        Spacer(1, 5 * mm),
        screenshot("calendar-v1.png", 145 * mm),
        Spacer(1, 4 * mm),
        card("コピーされる内容", "予定名、期間、終日／時刻、場所、メモ、色、繰り返し条件をコピーします。複数日予定では日数を保ったまま、貼り付け先を新しい開始日にします。Ctrl＋ドラッグした繰り返し予定の1回分は、独立した通常予定としてコピーします。", styles, SAND_PALE),
        Spacer(1, 3 * mm),
        p("操作の使い分け", styles["h2"]),
        data_table([
            ("同じ内容を複数日に登録", "予定を右クリックしてコピーし、任意の日付を右クリックして貼り付け"),
            ("予定日を変更", "通常のドラッグ"),
            ("元を残して別日にも登録", "Ctrlキーを押したままドラッグ"),
            ("一部を直してから登録", "予定をコピー後、「予定を追加」画面で「内容を貼り付け」して編集"),
        ], styles),
        PageBreak(),
    ])

    # 7: recurrence setup
    page_title(story, styles, "RECURRENCE", "繰り返し予定を設定する", "周期、間隔、曜日、終了条件を組み合わせて、定期予定と記念日を登録できます。")
    story.extend([
        screenshot("recurrence-v1.png", 145 * mm),
        Spacer(1, 4 * mm),
        data_table([
            ("毎日", "1日ごと、2日ごとなど。連日の当番や服薬予定"),
            ("毎週", "複数曜日を選択可能。月・水・金、隔週など"),
            ("毎月", "同じ日付、または第何週の同じ曜日"),
            ("毎年", "誕生日、記念日、更新日。2月29日はうるう年だけ表示"),
            ("終了条件", "終了なし、指定日まで、指定回数"),
        ], styles, 37 * mm),
        Spacer(1, 3 * mm),
        card("複数日と繰り返しの組み合わせ", "開始日から終了日までの日数も各回へ引き継ぎます。たとえば毎月1日から3日までの出張を登録すると、毎回3日間の予定として表示します。", styles, GREEN_PALE),
        PageBreak(),
    ])

    # 8: recurrence operations and agenda
    page_title(story, styles, "RECURRENCE SCOPE", "1回だけ・全体の編集と記念日", "繰り返し予定を開くと、今回だけ変えるか、シリーズ全体を変えるかを選べます。")
    story.extend([
        two_cards([
            ("この予定のみ", "選んだ回だけ内容や日付を変更します。削除すると、その回だけを除外します。通常ドラッグも、その回だけを移動します。"),
            ("繰り返し全体", "元の予定名、時刻、期間、周期、色などを更新します。削除すると、過去・未来の表示と個別例外を含むシリーズ全体を削除します。"),
        ], styles, [SKY_PALE, PURPLE_PALE]),
        Spacer(1, 4 * mm),
        screenshot("agenda.png", 118 * mm),
        Spacer(1, 3 * mm),
        p("日の予定一覧", styles["h2"]),
        p("予定がある日付を左クリックすると、その日の予定を一覧表示します。同じ日に5件以上あってもここですべて確認でき、予定を選ぶと編集できます。", styles["body"]),
        p("誕生日・記念日", styles["h2"]),
        bullet("繰り返し周期で「毎年」を選び、登録する月日を開始日にします。", styles),
        bullet("特定の年だけ内容を変える場合は「この予定のみ」、今後も含めて変える場合は「繰り返し全体」を選びます。", styles),
        bullet("記念日全体を削除すると、過去・未来のすべての年度表示から消えます。削除前に必要ならdataフォルダーをバックアップしてください。", styles),
        card("Googleから取り込んだ複雑な繰り返し", "Google独自の繰り返し条件はKoyomadoで表示・同期できます。周期そのものの変更がロックされている場合はGoogleカレンダー側で変更してください。個別回の編集はKoyomadoでも行えます。", styles, SAND_PALE),
        PageBreak(),
    ])

    # 9: appearance
    page_title(story, styles, "APPEARANCE", "背景・サイドバー・ウィンドウ", "デスクトップに馴染む8つの背景と、置き方に合わせた2段階の最小幅を用意しています。")
    story.extend([
        screenshot("settings-v1.png", 145 * mm),
        Spacer(1, 3 * mm),
        p("8つの背景テーマ", styles["h2"]),
        p("朝もや / 森の息吹 / 藤の夕暮れ / 陽だまり / 月夜の水面 / 空のそよ風 / 桜かすみ / 白樺の朝", styles["body"]),
        two_cards([
            ("サイドバー表示中", "今日と直近7日間、背景テーマのショートカットを表示。最小幅は806pxです。"),
            ("サイドバー非表示", "カレンダーをコンパクトに表示。最小幅は375pxです。開き直すときは必要な幅まで自動で広がります。"),
        ], styles, [GREEN_PALE, SKY_PALE]),
        Spacer(1, 3 * mm),
        card("モニター構成ごとに位置を記憶", "移動・サイズ変更・サイドバーの開閉状態を自動保存します。3画面、2画面など構成ごとに最後の位置を分けて記憶するため、以前の構成へ戻ると、その構成で保存した位置へ戻ります。画面外の位置は使わず、見える位置へ戻します。", styles, PURPLE_PALE),
        PageBreak(),
    ])

    # 10: taskbar, tray and autostart
    page_title(story, styles, "WINDOW DISPLAY", "タスクバー・トレイ・自動起動", "右上の歯車から、普段の使い方に合う表示先を選べます。初期設定はタスクバーのみです。")
    story.extend([
        data_table([
            ("タスクバーのみ（標準）", "最小化するとタスクバーへ残ります。×で閉じるとアプリを終了。トレイアイコンは表示しません。"),
            ("タスクトレイのみ", "最小化または×で画面を隠します。タスクバーには残らず、トレイアイコンから再表示・終了します。"),
            ("両方", "起動中はタスクバーとトレイの両方へ表示。最小化はタスクバー、×では画面を隠してトレイへ残します。"),
        ], styles, 53 * mm),
        Spacer(1, 4 * mm),
        two_cards([
            ("トレイから再表示", "アイコンを左クリックするか、右クリックして「カレンダーを表示」。完全終了は右クリックの「終了」。"),
            ("表示先を変えた直後", "タスクバー・トレイの表示をすぐに切り替えます。予定とウィンドウ位置は変わりません。"),
        ], styles, [GREEN_PALE, SKY_PALE]),
        Spacer(1, 5 * mm),
        p("Windows起動時に自動起動", styles["h2"]),
        step(1, "右上の歯車を開く", "「表示と起動の設定」を開きます。", styles),
        step(2, "自動起動をON", "「Windows起動時に自動起動」のスイッチを選びます。次回のWindowsサインイン時から起動します。", styles),
        step(3, "フォルダーを移動するときは登録し直す", "移動前に自動起動をOFFにし、移動後のkoyomado.exeから再びONにします。", styles),
        card("起動したのに見えないとき", "表示先がトレイを含む場合は、通知領域と「隠れているインジケーター」を確認します。保存位置が現在のモニター構成の画面外なら、Koyomadoは見える位置へ自動的に戻します。", styles, SAND_PALE),
        PageBreak(),
    ])

    # 11: data and update
    page_title(story, styles, "DATA AND UPDATE", "データ保存・持ち運び・更新", "予定と設定は暗号化せず、koyomado.exeと同じ場所のdataフォルダーへ保存します。Googleの更新トークンだけはWindows資格情報マネージャーへ保存します。")
    story.extend([
        data_table([
            ("calendar-data.json", "予定、開始・終了日時、繰り返し、削除済み予定、外観、表示先、Google接続設定"),
            ("calendar-data.backup.json", "予定データを更新する直前のバックアップ"),
            ("calendar-data.v1/v2.backup.json", "旧形式からversion 3へ移行する前の予定データ（移行時のみ）"),
            ("window-state.json", "モニター構成ごとのウィンドウ位置とサイズ"),
            ("window-state.backup.json", "位置情報を更新する直前のバックアップ"),
            ("window-state.v1.backup.json", "旧形式から移行する前の位置情報（移行時のみ）"),
            ("Windows資格情報", "Googleの更新トークン。PCごとに保存され、フォルダーには含まれません"),
        ], styles, 58 * mm),
        Spacer(1, 4 * mm),
        two_cards([
            ("USBメモリ", "Koyomadoフォルダー全体をコピーします。取り外す前にKoyomadoを完全終了してください。"),
            ("Google Drive", "同期完了後に起動し、同じフォルダーを複数PCから同時に開かないでください。競合の自動解決は行いません。"),
        ], styles, [GREEN_PALE, SKY_PALE]),
        Spacer(1, 4 * mm),
        p("新しい版へ更新する", styles["h2"]),
        step(1, "Koyomadoを終了", "タスクバーのみなら×で終了。トレイを使う設定なら、トレイアイコンを右クリックして「終了」を選びます。", styles),
        step(2, "dataをバックアップ", "現在のKoyomadoフォルダー内のdataフォルダーを、別の安全な場所へコピーします。", styles),
        step(3, "新しいZIPを展開", "新しいフォルダーへ「すべて展開」します。", styles),
        step(4, "dataを引き継ぐ", "古いKoyomadoフォルダーのdataフォルダーを、新しいKoyomadoフォルダーへコピーします。", styles),
        step(5, "起動して確認", "koyomado.exeを起動し、予定・背景・位置を確認します。自動起動は新しい場所からONにし直し、Google連携は移動先PCで再認証します。", styles),
        Spacer(1, 3 * mm),
        card("大切な注意", "dataフォルダーを削除したり、新しい空のdataだけを残したりすると、予定を引き継げません。アプリの更新前には必ずフォルダーごとバックアップしてください。", styles, ROSE_PALE),
        PageBreak(),
    ])

    # 12: Google overview
    page_title(story, styles, "GOOGLE CALENDAR", "Google連携のしくみ", "Google連携は任意機能で、初期状態はOFFです。利用者が有効にした場合だけGoogleへ通信します。")
    story.extend([
        two_cards([
            ("連携しない", "Koyomadoは従来どおり完全にローカルで動作します。Googleへの通信、ログイン、API設定は不要です。"),
            ("連携する", "利用者自身のGoogle CloudプロジェクトとOAuthクライアントを使い、選んだカレンダーと双方向同期します。"),
        ], styles, [GREEN_PALE, SKY_PALE]),
        Spacer(1, 4 * mm),
        p("同期する内容", styles["h2"]),
        data_table([
            ("予定", "予定名、開始・終了日時、終日、場所、メモ、繰り返し、削除"),
            ("複数日", "終日の連休・出張、日をまたぐ時刻付き予定を維持"),
            ("同期先", "ローカルのみ、特定アカウント、接続中の全アカウントから予定ごとに選択"),
            ("アカウント", "最大3件。各アカウントで同期するカレンダーを1つ選択"),
            ("予定の色", "Koyomado内の見た目として保持。Googleの色とは同期しません"),
        ], styles, 37 * mm),
        Spacer(1, 4 * mm),
        p("データと認証情報", styles["h2"]),
        bullet("予定はGoogle Calendar APIと利用者のPC間で直接送受信し、Y-TECのサーバーを経由しません。", styles),
        bullet("OAuthクライアントID、クライアントシークレット、プロジェクトIDはcalendar-data.jsonへ保存します。", styles),
        bullet("Googleの更新トークンはWindows資格情報マネージャーへ保存し、ポータブルフォルダーやGoogle Driveには入れません。", styles),
        card("通信を止める", "歯車でGoogleカレンダー連携をOFFにすると自動同期を停止します。アカウントの「接続解除」では認証情報と同期リンクを削除し、取り込み済み予定はローカル予定として残します。", styles, PURPLE_PALE),
        PageBreak(),
    ])

    # 13: Google Cloud project and API
    page_title(story, styles, "GOOGLE CLOUD 1", "プロジェクト作成とCalendar API", "以下は2026年8月時点のGoogle Cloud画面名です。表示名が変わった場合は、近い名称の項目を選んでください。")
    story.extend([
        step(1, "Google Cloud Consoleを開く", f'<link href="{GOOGLE_CONSOLE_URL}" color="#5f5278">{GOOGLE_CONSOLE_URL}</link>へ、連携に使うGoogleアカウントでログインします。', styles),
        step(2, "プロジェクトを作成", "上部のプロジェクト選択を開き、「新しいプロジェクト」を選びます。名前は例としてKoyomado Personalとし、作成後にそのプロジェクトへ切り替えます。", styles),
        step(3, "APIライブラリを開く", "左上のメニューから「APIとサービス」-「ライブラリ」を開きます。新しいGoogle Auth Platform画面では「APIs」または検索欄から進める場合があります。", styles),
        step(4, "Google Calendar APIを有効化", "Google Calendar APIを検索して選び、「有効にする」を押します。似た名前のCalDAV APIではありません。", styles),
        Spacer(1, 4 * mm),
        card("APIキーは作りません", "Koyomadoが使うのはAPIキーではなく、デスクトップアプリ用のOAuth 2.0クライアントです。利用者自身のプロジェクトを使うため、Y-TEC共通キーやY-TECへのAPI利用料はありません。Google Cloudの規約、割り当て、ほかに有効化したサービスの費用は利用者自身で管理してください。", styles, SAND_PALE),
        Spacer(1, 4 * mm),
        p("確認ポイント", styles["h2"]),
        bullet("画面上部のプロジェクト名が、今作成したKoyomado用プロジェクトになっている。", styles),
        bullet("Google Calendar APIの画面に「APIが有効です」または「管理」と表示される。", styles),
        bullet("組織のGoogle Workspace管理者が外部アプリを制限している場合は、管理者の許可が必要になることがあります。", styles),
        PageBreak(),
    ])

    # 14: Google OAuth consent
    page_title(story, styles, "GOOGLE CLOUD 2", "同意画面・利用者・権限を設定", "Google Auth Platformで、誰が使うかとKoyomadoへ許可する範囲を設定します。")
    story.extend([
        step(1, "Brandingを設定", "Google Auth Platformの「Branding」を開き、アプリ名にKoyomado、ユーザーサポートメールとデベロッパー連絡先に自分のメールアドレスを設定して保存します。", styles),
        step(2, "Audienceを設定", "個人のGoogleアカウントを使う場合は通常「External」を選びます。Google Workspace組織内だけで使う場合は、管理者方針に応じてInternalを選べることがあります。", styles),
        step(3, "テスト利用者を追加", "公開ステータスがTestingの間は「Test users」へ、Koyomadoと接続するGoogleアカウントを追加します。最大3アカウントを使う場合は3件とも追加します。", styles),
        step(4, "Data Accessの権限を確認", "次の4つを使用します。画面でスコープ追加が求められる場合は、必要なものだけを選びます。", styles),
        p("openid<br/>email<br/>https://www.googleapis.com/auth/calendar.events<br/>https://www.googleapis.com/auth/calendar.calendarlist.readonly", styles["code"]),
        Spacer(1, 3 * mm),
        two_cards([
            ("Testingの注意", "ExternalアプリをTestingのまま使うと、Googleの仕様により認証が約7日で期限切れになり、Koyomadoで再認証が必要になります。"),
            ("In productionの注意", "個人用でも公開ステータスをIn productionへ変更できますが、Calendar権限では未確認アプリの警告や検証案内が表示される場合があります。画面の内容を読んで判断してください。"),
        ], styles, [ROSE_PALE, SAND_PALE]),
        PageBreak(),
    ])

    # 15: OAuth client and Koyomado connection
    page_title(story, styles, "GOOGLE CLOUD 3", "Desktopクライアントを作って接続", "OAuthクライアントJSONをダウンロードし、Koyomadoへ読み込みます。JSONは公開・共有しないでください。")
    story.extend([
        step(1, "Clientsを開く", "Google Auth Platformの「Clients」から「Create Client」を選びます。旧画面では「APIとサービス」-「認証情報」-「認証情報を作成」-「OAuthクライアントID」です。", styles),
        step(2, "Desktop appを選ぶ", "Application typeで「Desktop app」を選び、名前をKoyomado Desktopなどにして作成します。Web applicationは選びません。", styles),
        step(3, "JSONをダウンロード", "作成したクライアントのダウンロードボタンからJSONを保存します。保存場所は後で分かる場所にします。", styles),
        step(4, "Koyomadoへ読み込む", "Koyomadoの歯車でGoogleカレンダー連携をONにし、「JSONを選択」を押して先ほどのファイルを選びます。設定後は「JSONを読み直す」と表示されます。", styles),
        step(5, "アカウントを接続", "「アカウントを接続」を押すと既定ブラウザーが開きます。Googleアカウントを選び、表示された権限を確認して許可します。Koyomadoへ戻るまでブラウザーを閉じないでください。", styles),
        Spacer(1, 3 * mm),
        screenshot("google-settings-v1.png", 132 * mm),
        Spacer(1, 2 * mm),
        p(f'<link href="{GOOGLE_CREDENTIALS_URL}" color="#5f5278">Google公式: デスクトップアプリの認証情報を作成</link>', styles["link"]),
        PageBreak(),
    ])

    # 16: account and sync operation
    page_title(story, styles, "GOOGLE SYNC", "アカウント・同期先・同期操作", "接続後に対象カレンダーを確認し、予定ごとの送信先を選びます。")
    story.extend([
        step(1, "同期するカレンダーを選ぶ", "接続直後にアカウントの「同期するカレンダー」を選びます。最初の同期後は誤結合を防ぐため選択をロックします。変更するときは接続解除後に接続し直します。", styles),
        step(2, "アカウントの同期をON", "アカウントカードの「このアカウントと同期」をONにします。不要なアカウントだけ一時停止できます。", styles),
        step(3, "予定の保存先を選ぶ", "予定追加・編集画面で、ローカルのみ、特定アカウント、または「すべて選択」を選びます。Googleから取得した予定は元アカウントへの同期を解除できません。", styles),
        step(4, "初回同期を確認", "設定画面の「今すぐ同期」を押し、最終同期日時とエラー表示を確認します。合成のテスト予定を双方で1件ずつ作り、往復することを確かめてから実予定に使うと安全です。", styles),
        Spacer(1, 4 * mm),
        p("自動同期のタイミング", styles["h2"]),
        data_table([
            ("起動・再表示", "アプリ起動時、トレイなどから画面へ戻したとき"),
            ("定期", "アプリが表示されている間、約60秒ごと"),
            ("予定変更直後", "追加、編集、削除、ドラッグ移動、コピー後"),
            ("手動", "上部の同期ボタン、または設定の「今すぐ同期」"),
        ], styles, 43 * mm),
        Spacer(1, 4 * mm),
        card("別のPCへ移した場合", "予定とGoogle接続設定はフォルダーと一緒に移動しますが、更新トークンは移動しません。移動先で「再認証」を押し、各アカウントを認証し直してください。同じGoogle Driveフォルダーを複数PCで同時起動しないでください。", styles, SKY_PALE),
        PageBreak(),
    ])

    # 17: Google conflicts and troubleshooting
    page_title(story, styles, "GOOGLE TROUBLE", "競合・解除・接続トラブル", "同期で勝手に予定を消さないため、両側編集を検出したときは内容を2件に分けて残します。")
    story.extend([
        two_cards([
            ("競合が起きた", "同じ予定をKoyomadoとGoogleで前回同期後に変更すると、両方の内容を別予定として保存し、編集画面に競合表示を出します。内容を確認し、必要な方を残してください。"),
            ("接続を解除", "アカウントカードの「接続解除」を選びます。Googleへのトークン失効を試み、Windows資格情報と同期リンクを削除します。取り込み済み予定はローカルへ残ります。"),
        ], styles, [ROSE_PALE, GREEN_PALE]),
        Spacer(1, 4 * mm),
        data_table([
            ("7日ほどで再認証", "OAuth公開ステータスがTestingの可能性。Audienceと公開ステータスを確認"),
            ("未確認アプリの警告", "自分で作成したプロジェクト名・Googleアカウント・要求権限を確認。心当たりがなければ中止"),
            ("JSONを読めない", "Desktop app用JSONか確認。APIキーのJSONやWeb application用は使用不可"),
            ("ブラウザー後に戻らない", "Koyomadoを開いたまま再試行。ファイアウォールやセキュリティ製品のlocalhost通信も確認"),
            ("カレンダーを変えたい", "いったん接続解除し、再接続直後に対象カレンダーを選び直す"),
            ("予定が同期されない", "Google連携、アカウント同期、予定の同期先を確認して「今すぐ同期」"),
        ], styles, 48 * mm),
        Spacer(1, 4 * mm),
        card("安全と費用", "OAuth JSON、トークン、実予定を第三者へ送らないでください。Koyomadoは課金設定を追加せず、Y-TECの共通APIも使いません。ただしGoogle Cloudアカウント全体の設定と利用規約は利用者の責任で管理してください。", styles, SAND_PALE),
        Spacer(1, 4 * mm),
        p(f'<link href="{GOOGLE_USER_DATA_URL}" color="#5f5278">Google API Services User Data Policy</link>', styles["link"]),
        PageBreak(),
    ])

    # 18: trouble and reference
    page_title(story, styles, "HELP", "困ったとき・早見表", "画面に予定が見えない場合は、まず月、日付、タスクバーまたはトレイ、dataフォルダーの順に確認してください。")
    story.extend([
        data_table([
            ("画面が見つからない", "タスクバーを確認。トレイを使う設定ではKoyomadoアイコンを左クリックし、隠れているアイコンも確認します。"),
            ("予定が見えない", "表示中の年月を確認し、予定のある日付をクリックして日の一覧を開きます。"),
            ("記念日が翌年に出ない", "繰り返し周期が「毎年」か確認。2月29日はうるう年だけ表示されます。"),
            ("終了日時が戻った", "開始時刻を変えると終了は1時間後へ再設定されます。開始を決めた後に終了日時を変更します。"),
            ("貼り付けが選べない", "先に予定を右クリックして「内容をコピー」を選びます。"),
            ("起動時の位置がおかしい", "現在のモニター構成で最後に保存した位置へ戻ります。初めての構成や画面外の位置は自動で見える位置へ戻るため、希望の場所へ移動して終了し直してください。"),
            ("データが壊れた", "バックアップから自動復旧を試み、壊れたファイルはcorrupt付きの名前で退避します。直らない場合はdataのバックアップを戻します。"),
        ], styles, 46 * mm),
        Spacer(1, 4 * mm),
        p("操作早見表", styles["h2"]),
        two_cards([
            ("左クリック", "予定: 編集<br/>空の日付: 予定を追加<br/>予定がある日付: 日の一覧<br/>トレイアイコン: 再表示"),
            ("右クリック", "予定: コピー・編集・削除<br/>日付: 貼り付け・追加<br/>トレイ: 表示・終了"),
            ("ドラッグ", "通常: 予定を移動<br/>Ctrlを押しながら: 予定をコピー"),
        ], styles, [PURPLE_PALE, GREEN_PALE, SKY_PALE]),
        Spacer(1, 4 * mm),
        p("仕様上の範囲", styles["h2"]),
        p("Koyomadoには、印刷、予定のPDF出力、アクセス解析、独自クラウドサーバーはありません。外部通信は利用者が任意で有効にするGoogleカレンダー連携だけです。予定データとOAuthクライアント設定は暗号化されないため、機密情報やパスワードの保存には使用しないでください。祝日は1970年から2050年までの内蔵データを使います。", styles["body"]),
        card("お問い合わせ前に用意すると役立つ情報", "Koyomadoのバージョン、Windowsのバージョン、発生した操作、表示されたメッセージ、再現手順。予定の本文や個人情報は送らないでください。", styles, SAND_PALE),
        Spacer(1, 5 * mm),
        p(f'<b>公式ページ</b><br/><link href="{OFFICIAL_URL}" color="#5f5278">{OFFICIAL_URL}</link><br/><br/><b>ソースコード</b><br/><link href="{SOURCE_URL}" color="#5f5278">{SOURCE_URL}</link><br/><br/>利用条件はApache License 2.0のLICENSE.txt、Google連携の扱いはPRIVACY.mdをご確認ください。', styles["link"]),
    ])
    return story


def generate(output: Path) -> None:
    register_fonts()
    styles = build_styles()
    output.parent.mkdir(parents=True, exist_ok=True)
    doc = BaseDocTemplate(
        str(output),
        pagesize=A4,
        leftMargin=MARGIN_X,
        rightMargin=MARGIN_X,
        topMargin=MARGIN_TOP,
        bottomMargin=MARGIN_BOTTOM,
        title=f"Koyomado 操作説明書 v{VERSION}",
        author="Y-TEC",
        subject="Koyomado Windowsポータブルカレンダーの操作説明書",
        creator="Koyomado manual generator",
    )
    frame = Frame(
        doc.leftMargin,
        doc.bottomMargin,
        doc.width,
        doc.height,
        id="content",
        leftPadding=0,
        rightPadding=0,
        topPadding=0,
        bottomPadding=0,
    )
    doc.addPageTemplates([PageTemplate(id="main", frames=[frame], onPage=decorate_page)])
    doc.build(build_story(styles))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        type=Path,
        default=ROOT / "docs" / "Koyomado操作説明書.pdf",
        help="出力するPDFファイル",
    )
    args = parser.parse_args()
    generate(args.output.resolve())
    print(f"操作説明書を作成しました: {args.output.resolve()}")


if __name__ == "__main__":
    main()
