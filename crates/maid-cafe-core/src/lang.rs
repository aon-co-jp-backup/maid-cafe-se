//! 読み上げの言語。固定のセリフと定型文を10言語で内蔵する(翻訳サービスは使わない=通信もAPIキーも不要)。
//!
//! **翻訳はAI(Claude)が下訳したもので、各言語のネイティブによる確認はしていない。** 自由に入力した文章は翻訳しない
//! (入力したまま、その言語の音声で読み上げる)。

use crate::model::VoiceStyle;

/// 対応言語(コード, 自国語の名前, 英語名)。先頭が既定の日本語。
pub const LANGS: [(&str, &str, &str); 10] = [
    ("ja", "日本語", "Japanese"),
    ("en", "English", "English"),
    ("zh", "中文(简体)", "Chinese"),
    ("ko", "한국어", "Korean"),
    ("it", "Italiano", "Italian"),
    ("fr", "Français", "French"),
    ("de", "Deutsch", "German"),
    ("ru", "Русский", "Russian"),
    ("fa", "فارسی", "Persian (Iran)"),
    ("ar", "العربية", "Arabic"),
];

pub const DEFAULT_LANG: &str = "ja";

pub fn is_known_lang(code: &str) -> bool {
    LANGS.iter().any(|l| l.0 == code)
}

/// 未知のコードは日本語にする。
pub fn normalize(code: &str) -> &'static str {
    LANGS.iter().find(|l| l.0 == code).map(|l| l.0).unwrap_or(DEFAULT_LANG)
}

/// 言語コードの位置(表の並び)。
fn idx(code: &str) -> usize {
    LANGS.iter().position(|l| l.0 == code).unwrap_or(0)
}

/// 固定セリフの、その言語での読み上げ文。`ja`は`MaidPhrases`側(長音などTTS向けの表記)を使うので、ここは日本語以外。
/// 並びは`LANGS`と同じ(ja, en, zh, ko, it, fr, de, ru, fa, ar)。日本語の位置は空。
const PHRASES: [(&str, [&str; 10]); 44] = [
    (
        "okite",
        [
            "",
            "Master, wake up, please. Do your best today too!",
            "主人，起床啦。今天也要加油哦！",
            "주인님, 일어나세요. 오늘도 힘내세요!",
            "Padrone, svegliati. Anche oggi fai del tuo meglio!",
            "Maître, réveillez-vous. Courage pour aujourd'hui aussi !",
            "Herr, aufwachen. Gib auch heute dein Bestes!",
            "Господин, проснитесь. И сегодня у вас всё получится!",
            "ارباب، بیدار شوید. امروز هم تلاش کنید!",
            "سيدي، استيقظ. اجتهد اليوم أيضًا!",
        ],
    ),
    (
        "okaeri",
        [
            "",
            "Welcome home, Master!",
            "欢迎回来，主人！",
            "어서 오세요, 주인님!",
            "Bentornato a casa, Padrone!",
            "Bienvenue à la maison, Maître !",
            "Willkommen zu Hause, Herr!",
            "Добро пожаловать домой, господин!",
            "به خانه خوش آمدید، ارباب!",
            "مرحبًا بعودتك إلى المنزل يا سيدي!",
        ],
    ),
    (
        "oishiku",
        [
            "",
            "Make it delicious! Moe moe kyun!",
            "变得好吃吧！萌萌哒！",
            "맛있어져라! 모에모에 큥!",
            "Diventa delizioso! Moe moe kyun!",
            "Deviens délicieux ! Moe moe kyun !",
            "Werde lecker! Moe moe kyun!",
            "Стань вкусным! Мое-мое кюн!",
            "خوشمزه شو! موئه موئه کیون!",
            "كن لذيذًا! موي موي كيون!",
        ],
    ),
    (
        "meh",
        [
            "",
            "No! That's not allowed! Stop dwelling on it, cheer up with your maid, and let's do our best!",
            "不行哦！不要总是闷闷不乐，和女仆一起打起精神加油吧！",
            "안 돼요! 언제까지 끙끙 앓고 있을 거예요? 메이드와 함께 힘을 내서 열심히 해봐요!",
            "No! Non va bene! Smettila di abbatterti e riprendi coraggio insieme alla tua cameriera, forza!",
            "Non ! Ce n'est pas bien ! Arrêtez de ruminer et retrouvez de l'énergie avec votre servante, allons-y !",
            "Nein! Das geht nicht! Grübele nicht ewig, nimm mit deinem Dienstmädchen neuen Mut, und los geht's!",
            "Нет! Так нельзя! Не унывайте, соберитесь вместе с вашей горничной, и вперёд!",
            "نه! این درست نیست! دیگر غصه نخور و با خدمتکارت انرژی بگیر و تلاش کنیم!",
            "لا! هذا لا يجوز! كفّ عن الحزن وتشجّع مع خادمتك، هيا نبذل جهدنا!",
        ],
    ),
    (
        "fight",
        [
            "",
            "Fight! Fight!",
            "加油！加油！",
            "파이팅! 파이팅!",
            "Forza! Forza!",
            "Courage ! Courage !",
            "Los geht's! Los geht's!",
            "Вперёд! Вперёд!",
            "تلاش کن! تلاش کن!",
            "هيا! هيا!",
        ],
    ),
    (
        "excellent",
        ["", "Excellent!", "太棒了！", "훌륭해요!", "Eccellente!", "Excellent !", "Ausgezeichnet!", "Отлично!", "عالی!", "ممتاز!"],
    ),
    (
        "perfect",
        ["", "Perfect!", "完美！", "완벽해요!", "Perfetto!", "Parfait !", "Perfekt!", "Идеально!", "کامل!", "مثالي!"],
    ),
    (
        "kusuri",
        ["", "It's time for your medicine.", "该吃药的时间啦。", "약 먹을 시간이에요.", "È l'ora della medicina.", "C'est l'heure de prendre vos médicaments.", "Es ist Zeit für Ihre Medizin.", "Пора принимать лекарство.", "وقت مصرف دارو است.", "حان وقت الدواء."],
    ),
    (
        "megusuri",
        ["", "It's time for your eye drops.", "该点眼药水的时间啦。", "안약 넣을 시간이에요.", "È l'ora delle gocce per gli occhi.", "C'est l'heure des gouttes pour les yeux.", "Es ist Zeit für Ihre Augentropfen.", "Пора закапать глаза.", "وقت قطره چشم است.", "حان وقت قطرة العين."],
    ),
    (
        "gomi",
        ["", "It's time to take out the trash.", "该倒垃圾的时间啦。", "쓰레기 버릴 시간이에요.", "È ora di buttare la spazzatura.", "C'est l'heure de sortir les poubelles.", "Es ist Zeit, den Müll rauszubringen.", "Пора выносить мусор.", "وقت بیرون بردن زباله است.", "حان وقت إخراج القمامة."],
    ),
    (
        "sentaku",
        ["", "It's time to do the laundry.", "该洗衣服的时间啦。", "빨래할 시간이에요.", "È ora di fare il bucato.", "C'est l'heure de faire la lessive.", "Es ist Zeit, die Wäsche zu waschen.", "Пора стирать бельё.", "وقت لباس‌شویی است.", "حان وقت غسل الملابس."],
    ),
    (
        "torikomi",
        ["", "It's time to bring in the laundry.", "该收衣服的时间啦。", "빨래를 걷을 시간이에요.", "È ora di ritirare il bucato.", "C'est l'heure de rentrer le linge.", "Es ist Zeit, die Wäsche hereinzuholen.", "Пора снимать бельё.", "وقت جمع کردن لباس‌ها است.", "حان وقت جمع الملابس."],
    ),
    (
        "okuri",
        ["", "It's time to take the kids to school.", "该送孩子的时间啦。", "아이를 데려다줄 시간이에요.", "È ora di accompagnare i bambini.", "C'est l'heure d'emmener les enfants.", "Es ist Zeit, die Kinder zu bringen.", "Пора отвозить детей.", "وقت رساندن بچه‌ها است.", "حان وقت توصيل الأطفال."],
    ),
    (
        "mukae",
        ["", "It's time to pick up the kids.", "该去接孩子的时间啦。", "아이를 데리러 갈 시간이에요.", "È ora di andare a prendere i bambini.", "C'est l'heure d'aller chercher les enfants.", "Es ist Zeit, die Kinder abzuholen.", "Пора забирать детей.", "وقت رفتن دنبال بچه‌ها است.", "حان وقت اصطحاب الأطفال."],
    ),
    (
        "shippu",
        ["", "It's time to change your compress.", "该贴膏药的时间啦。", "파스 붙일 시간이에요.", "È ora di mettere il cerotto.", "C'est l'heure de poser votre compresse.", "Es ist Zeit für Ihr Schmerzpflaster.", "Пора наклеить пластырь.", "وقت چسباندن چسب مسکن است.", "حان وقت لصقة الألم."],
    ),
    (
        "chusha",
        ["", "It's time for your injection.", "该打针的时间啦。", "주사 맞을 시간이에요.", "È l'ora dell'iniezione.", "C'est l'heure de votre piqûre.", "Es ist Zeit für Ihre Spritze.", "Пора делать укол.", "وقت آمپول است.", "حان وقت الحقنة."],
    ),
    (
        "patrol",
        ["", "It's time for the patrol.", "该巡逻的时间啦。", "순찰할 시간이에요.", "È l'ora del pattugliamento.", "C'est l'heure de la patrouille.", "Es ist Zeit für die Streife.", "Пора на патрулирование.", "وقت گشت‌زنی است.", "حان وقت الدورية."],
    ),
    (
        "kaigi",
        ["", "It's time for the meeting.", "该开会的时间啦。", "회의할 시간이에요.", "È ora della riunione.", "C'est l'heure de la réunion.", "Es ist Zeit für die Besprechung.", "Пора на совещание.", "وقت جلسه است.", "حان وقت الاجتماع."],
    ),
    (
        "uchiawase",
        ["", "It's time for the discussion.", "该碰头商量的时间啦。", "미팅할 시간이에요.", "È ora dell'incontro.", "C'est l'heure de la réunion de travail.", "Es ist Zeit für die Absprache.", "Пора на планёрку.", "وقت جلسه هماهنگی است.", "حان وقت اجتماع التنسيق."],
    ),
    (
        "omutsu",
        ["", "It's time to change the diaper.", "该换尿布的时间啦。", "기저귀 갈 시간이에요.", "È ora di cambiare il pannolino.", "C'est l'heure de changer la couche.", "Es ist Zeit, die Windel zu wechseln.", "Пора менять подгузник.", "وقت عوض کردن پوشک است.", "حان وقت تغيير الحفاض."],
    ),
    (
        "milk",
        ["", "It's time for the milk.", "该喝奶的时间啦。", "우유 먹을 시간이에요.", "È l'ora del latte.", "C'est l'heure du lait.", "Es ist Zeit für die Milch.", "Пора давать молоко.", "وقت شیر است.", "حان وقت الحليب."],
    ),
    (
        "ryouri",
        ["", "It's time to cook.", "该做饭的时间啦。", "요리할 시간이에요.", "È ora di cucinare.", "C'est l'heure de cuisiner.", "Es ist Zeit zu kochen.", "Пора готовить.", "وقت آشپزی است.", "حان وقت الطبخ."],
    ),
    (
        "shokuji",
        ["", "It's time to eat.", "该吃饭的时间啦。", "식사할 시간이에요.", "È ora di mangiare.", "C'est l'heure de manger.", "Es ist Zeit zum Essen.", "Пора есть.", "وقت غذا است.", "حان وقت الطعام."],
    ),
    (
        "haizen",
        ["", "It's time to serve the meal.", "该上菜的时间啦。", "상 차릴 시간이에요.", "È ora di servire il pasto.", "C'est l'heure de servir le repas.", "Es ist Zeit, das Essen aufzutragen.", "Пора накрывать на стол.", "وقت چیدن غذا است.", "حان وقت تقديم الطعام."],
    ),
    (
        "sagezen",
        ["", "It's time to clear the table.", "该收拾餐具的时间啦。", "상 치울 시간이에요.", "È ora di sparecchiare.", "C'est l'heure de débarrasser la table.", "Es ist Zeit, den Tisch abzuräumen.", "Пора убирать со стола.", "وقت جمع کردن سفره است.", "حان وقت رفع المائدة."],
    ),
    (
        "sentakuki",
        ["", "It's time for the washing machine.", "该用洗衣机的时间啦。", "세탁기 돌릴 시간이에요.", "È ora della lavatrice.", "C'est l'heure de la machine à laver.", "Es ist Zeit für die Waschmaschine.", "Пора включать стиральную машину.", "وقت ماشین لباس‌شویی است.", "حان وقت الغسالة."],
    ),
    (
        "hosu",
        ["", "It's time to hang out the laundry.", "该晾衣服的时间啦。", "빨래를 널 시간이에요.", "È ora di stendere il bucato.", "C'est l'heure d'étendre le linge.", "Es ist Zeit, die Wäsche aufzuhängen.", "Пора развешивать бельё.", "وقت پهن کردن لباس‌ها است.", "حان وقت نشر الملابس."],
    ),
    (
        "okashi",
        ["", "It's snack time.", "该吃点心的时间啦。", "간식 시간이에요.", "È l'ora dei dolcetti.", "C'est l'heure du goûter.", "Es ist Zeit für Süßigkeiten.", "Пора пить чай со сладостями.", "وقت شیرینی است.", "حان وقت الحلوى."],
    ),
    (
        "itterassyai",
        ["", "Have a nice day, Master!", "主人，路上小心哦！", "다녀오세요, 주인님!", "Buona giornata, Padrone!", "Bonne journée, Maître !", "Einen schönen Tag, Herr!", "Хорошего дня, Господин!", "روز خوبی داشته باشید، ارباب!", "يومًا سعيدًا يا سيدي!"],
    ),
    (
        "ofuro",
        ["", "Would you like a bath? Or would you like me?", "要先洗澡吗？还是先选我呢？", "목욕하시겠어요? 아니면 저로 하시겠어요?", "Vuole fare il bagno? Oppure preferisce me?", "Voulez-vous un bain ? Ou plutôt moi ?", "Möchten Sie ein Bad? Oder lieber mich?", "Хотите ванну? Или меня?", "حمام می‌خواهید؟ یا من را؟", "هل تريد الاستحمام؟ أم تريدني؟"],
    ),
    (
        "itadakimasu",
        ["", "It's time for \"Itadakimasu\" before we eat.", "该说“我开动了”的时间啦。", "“잘 먹겠습니다” 할 시간이에요.", "È ora di dire «Itadakimasu» prima di mangiare.", "C'est l'heure de dire « Itadakimasu » avant de manger.", "Es ist Zeit für das „Itadakimasu“ vor dem Essen.", "Пора сказать «Итадакимас» перед едой.", "وقت گفتن «ایتاداکیماس» پیش از غذا است.", "حان وقت قول «إيتاداكيماس» قبل الطعام."],
    ),
    (
        "katsucurry",
        ["", "I want to eat delicious katsu curry~", "好想吃美味的炸猪排咖喱饭～", "맛있는 돈가스 카레가 먹고 싶어요~", "Voglio mangiare un buon curry con cotoletta~", "J'ai envie d'un délicieux curry katsu~", "Ich möchte ein leckeres Katsu-Curry essen~", "Хочу вкусное карри с кацу~", "دلم کاری کاتسوی خوشمزه می‌خواهد~", "أريد أن آكل كاري الكاتسو اللذيذ~"],
    ),
    (
        "kakifurai",
        ["", "I want to eat delicious fried oyster curry~", "好想吃美味的炸牡蛎咖喱饭～", "맛있는 굴튀김 카레가 먹고 싶어요~", "Voglio mangiare un buon curry con ostriche fritte~", "J'ai envie d'un délicieux curry aux huîtres frites~", "Ich möchte ein leckeres Curry mit frittierten Austern essen~", "Хочу вкусное карри с жареными устрицами~", "دلم کاری صدف سرخ‌شده خوشمزه می‌خواهد~", "أريد أن آكل كاري المحار المقلي اللذيذ~"],
    ),
    (
        "ramen",
        ["", "I want to eat delicious ramen~", "好想吃美味的拉面～", "맛있는 라멘이 먹고 싶어요~", "Voglio mangiare un buon ramen~", "J'ai envie d'un délicieux ramen~", "Ich möchte leckeres Ramen essen~", "Хочу вкусный рамен~", "دلم رامن خوشمزه می‌خواهد~", "أريد أن آكل الرامن اللذيذ~"],
    ),
    (
        "oyakodon",
        ["", "I want to eat a delicious oyakodon (chicken and egg rice bowl)~", "好想吃美味的亲子丼～", "맛있는 오야코동이 먹고 싶어요~", "Voglio mangiare un buon oyakodon (ciotola di riso con pollo e uova)~", "J'ai envie d'un délicieux oyakodon (bol de riz au poulet et à l'œuf)~", "Ich möchte eine leckere Oyakodon (Reisschüssel mit Huhn und Ei) essen~", "Хочу вкусный оякодон (рис с курицей и яйцом)~", "دلم اویاکودون خوشمزه (برنج با مرغ و تخم‌مرغ) می‌خواهد~", "أريد أن آكل أوياكودون اللذيذ (أرز بالدجاج والبيض)~"],
    ),
    (
        "yakiniku",
        ["", "I want to eat delicious Kuroge Wagyu yakiniku~", "好想吃美味的黑毛和牛烤肉～", "맛있는 구로게 와규 야키니쿠가 먹고 싶어요~", "Voglio mangiare un buon yakiniku di Wagyu Kuroge~", "J'ai envie d'un délicieux yakiniku de bœuf Wagyu Kuroge~", "Ich möchte leckeres Yakiniku vom Kuroge Wagyu essen~", "Хочу вкусное якинику из мраморной говядины вагю~", "دلم یاکینیکوی خوشمزه با گوشت واگیوی کوروگه می‌خواهد~", "أريد أن آكل ياكينيكو لحم الواغيو الكوروغي اللذيذ~"],
    ),
    (
        "beefstew",
        ["", "I want to eat delicious beef stew~", "好想吃美味的炖牛肉～", "맛있는 비프스튜가 먹고 싶어요~", "Voglio mangiare un buon stufato di manzo~", "J'ai envie d'un délicieux bœuf bourguignon~", "Ich möchte leckeren Rindereintopf essen~", "Хочу вкусное тушёное мясо с говядиной~", "دلم خورش گوشت گاو خوشمزه می‌خواهد~", "أريد أن آكل يخنة اللحم البقري اللذيذة~"],
    ),
    (
        "whitestew",
        ["", "I want to eat delicious white stew~", "好想吃美味的奶油炖菜～", "맛있는 화이트 스튜가 먹고 싶어요~", "Voglio mangiare un buon stufato bianco~", "J'ai envie d'un délicieux ragoût blanc à la crème~", "Ich möchte leckeren weißen Eintopf essen~", "Хочу вкусное сливочное рагу~", "دلم خورش سفید خوشمزه می‌خواهد~", "أريد أن آكل يخنة الكريمة البيضاء اللذيذة~"],
    ),
    (
        "omurice",
        ["", "I want to eat delicious omurice~", "好想吃美味的蛋包饭～", "맛있는 오므라이스가 먹고 싶어요~", "Voglio mangiare un buon omurice~", "J'ai envie d'un délicieux omurice~", "Ich möchte leckeres Omurice essen~", "Хочу вкусный омурайс~", "دلم املت‌برنج (اومورایس) خوشمزه می‌خواهد~", "أريد أن آكل الأومورايس اللذيذ~"],
    ),
    (
        "mazingerz",
        ["", "I want to watch Mazinger Z~", "好想看魔神Z～", "마징가 Z가 보고 싶어요~", "Voglio guardare Mazinga Z~", "J'ai envie de regarder Mazinger Z~", "Ich möchte Mazinger Z sehen~", "Хочу посмотреть «Мазингер Зет»~", "دلم می‌خواهد ماژینگر زد را ببینم~", "أريد أن أشاهد مازينجر زد~"],
    ),
    (
        "greatmazinger",
        ["", "I want to watch Great Mazinger~", "好想看大魔神～", "그레이트 마징가가 보고 싶어요~", "Voglio guardare Great Mazinga~", "J'ai envie de regarder Great Mazinger~", "Ich möchte Great Mazinger sehen~", "Хочу посмотреть «Грейт Мазингер»~", "دلم می‌خواهد گریت ماژینگر را ببینم~", "أريد أن أشاهد غريت مازينجر~"],
    ),
    (
        "grendizer",
        ["", "I want to watch UFO Robot Grendizer~", "好想看UFO机器人金刚战神～", "UFO 로봇 그렌다이저가 보고 싶어요~", "Voglio guardare UFO Robot Goldrake~", "J'ai envie de regarder Goldorak~", "Ich möchte Grendizer sehen~", "Хочу посмотреть «Грендайзер»~", "دلم می‌خواهد یو‌اف‌او روبو گرندایزر را ببینم~", "أريد أن أشاهد غرندايزر~"],
    ),
    (
        "gaiking",
        ["", "I want to watch Daikumaryu Gaiking~", "好想看大空魔龙盖基因～", "대공마룡 가이킹이 보고 싶어요~", "Voglio guardare Gaiking, il drago dei cieli~", "J'ai envie de regarder Gaiking~", "Ich möchte Gaiking sehen~", "Хочу посмотреть «Гайкинг»~", "دلم می‌خواهد گایکینگ را ببینم~", "أريد أن أشاهد غايكينغ~"],
    ),
    (
        "jeeg",
        ["", "I want to watch Steel Jeeg~", "好想看钢铁杰克～", "강철 지그가 보고 싶어요~", "Voglio guardare Jeeg Robot d'acciaio~", "J'ai envie de regarder Jeeg~", "Ich möchte Steel Jeeg sehen~", "Хочу посмотреть «Стальной Джиг»~", "دلم می‌خواهد جیگ فولادی را ببینم~", "أريد أن أشاهد جيغ الفولاذي~"],
    ),
];

/// 固定セリフの、日本語以外での読み上げ文。日本語(または未対応)なら`None`。
pub fn phrase_spoken(id: &str, lang: &str) -> Option<&'static str> {
    let i = idx(lang);
    if i == 0 {
        return None;
    }
    PHRASES.iter().find(|p| p.0 == id).map(|p| p.1[i]).filter(|s| !s.is_empty())
}

/// 指定時刻の基本メッセージ(`text`はユーザーの入力そのまま)。
const ALARM_MAID: [&str; 10] = [
    "",
    "Master, it's time. {t}. Please don't forget.",
    "主人，时间到了哦。{t}。请不要忘记哦",
    "주인님, 시간이에요. {t}. 잊지 말아 주세요",
    "Padrone, è l'ora. {t}. Non dimenticare, per favore",
    "Maître, il est l'heure. {t}. N'oubliez pas, s'il vous plaît",
    "Herr, es ist Zeit. {t}. Bitte vergiss es nicht",
    "Господин, пора. {t}. Пожалуйста, не забудьте",
    "ارباب، وقتش رسیده. {t}. لطفاً فراموش نکنید",
    "سيدي، حان الوقت. {t}. من فضلك لا تنسَ",
];
const ALARM_DEEP: [&str; 10] = [
    "",
    "Time. {t}",
    "时间到了。{t}",
    "시간이다. {t}",
    "È l'ora. {t}",
    "C'est l'heure. {t}",
    "Es ist Zeit. {t}",
    "Время. {t}",
    "وقتشه. {t}",
    "حان الوقت. {t}",
];
/// 予告(`{m}`=分、`{t}`=名前)。
const PRE_MAID: [&str; 10] = [
    "",
    "Master, {m} minutes until {t}.",
    "主人，还有{m}分钟就到{t}的时间了哦",
    "주인님, {m}분 후에 {t} 시간이에요",
    "Padrone, mancano {m} minuti a {t}",
    "Maître, il reste {m} minutes avant {t}",
    "Herr, noch {m} Minuten bis {t}",
    "Господин, до {t} осталось {m} минут",
    "ارباب، {m} دقیقه تا {t} مانده",
    "سيدي، تبقّى {m} دقيقة على {t}",
];
const PRE_DEEP: [&str; 10] = [
    "",
    "{m} minutes until {t}.",
    "还有{m}分钟，{t}的时间",
    "{m}분 후, {t} 시간이다",
    "Mancano {m} minuti a {t}",
    "Encore {m} minutes avant {t}",
    "Noch {m} Minuten bis {t}",
    "До {t} {m} минут",
    "{m} دقیقه تا {t}",
    "تبقّى {m} دقيقة على {t}",
];

/// 日本語以外の指定時刻の基本メッセージ。
pub fn alarm_message(text: &str, voice: VoiceStyle, lang: &str) -> Option<String> {
    let i = idx(lang);
    if i == 0 {
        return None;
    }
    let t = match voice {
        VoiceStyle::Maid => ALARM_MAID[i],
        VoiceStyle::DeepMale => ALARM_DEEP[i],
    };
    Some(t.replace("{t}", text.trim()))
}

/// 日本語以外の予告メッセージ。
pub fn pre_notice_message(title: &str, minutes: u32, voice: VoiceStyle, lang: &str) -> Option<String> {
    let i = idx(lang);
    if i == 0 {
        return None;
    }
    let t = match voice {
        VoiceStyle::Maid => PRE_MAID[i],
        VoiceStyle::DeepMale => PRE_DEEP[i],
    };
    Some(t.replace("{m}", &minutes.to_string()).replace("{t}", title))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_every_phrase_and_template() {
        for (i, (code, name, _)) in LANGS.iter().enumerate().skip(1) {
            assert!(!name.is_empty());
            for (id, texts) in PHRASES.iter() {
                assert!(!texts[i].is_empty(), "{id} が {code} に無い");
            }
            for table in [&ALARM_MAID, &ALARM_DEEP] {
                assert!(table[i].contains("{t}"), "{code}");
            }
            for table in [&PRE_MAID, &PRE_DEEP] {
                assert!(table[i].contains("{m}") && table[i].contains("{t}"), "{code}");
            }
        }
        assert!(PHRASES.iter().all(|p| p.1[0].is_empty()));
    }

    #[test]
    fn normalize_and_lookup() {
        assert_eq!("ja", normalize("xx"));
        assert_eq!("fa", normalize("fa"));
        assert_eq!(None, phrase_spoken("okite", "ja"));
        assert_eq!(Some("Perfect!"), phrase_spoken("perfect", "en"));
        assert_eq!(None, phrase_spoken("bogus", "en"));
        assert_eq!("Master, 30 minutes until Meeting.", pre_notice_message("Meeting", 30, VoiceStyle::Maid, "en").unwrap());
        assert_eq!("Time. pills", alarm_message(" pills ", VoiceStyle::DeepMale, "en").unwrap());
        assert_eq!(None, alarm_message("x", VoiceStyle::Maid, "ja"));
    }
}
