pub const LEN:usize = 1200;
pub const TAB: [f32; LEN] = [
0.0000000000,
0.0000017135,
0.0000068539,
0.0000154212,
0.0000274153,
0.0000428362,
0.0000616838,
0.0000839578,
0.0001096583,
0.0001387849,
0.0001713375,
0.0002073159,
0.0002467198,
0.0002895490,
0.0003358031,
0.0003854819,
0.0004385849,
0.0004951120,
0.0005550625,
0.0006184362,
0.0006852326,
0.0007554513,
0.0008290917,
0.0009061533,
0.0009866358,
0.0010705384,
0.0011578606,
0.0012486018,
0.0013427613,
0.0014403387,
0.0015413332,
0.0016457440,
0.0017535704,
0.0018648118,
0.0019794672,
0.0020975361,
0.0022190176,
0.0023439110,
0.0024722151,
0.0026039290,
0.0027390523,
0.0028775837,
0.0030195222,
0.0031648672,
0.0033136173,
0.0034657714,
0.0036213291,
0.0037802886,
0.0039426493,
0.0041084099,
0.0042775692,
0.0044501261,
0.0046260799,
0.0048054284,
0.0049881712,
0.0051743067,
0.0053638336,
0.0055567506,
0.0057530566,
0.0059527499,
0.0061558299,
0.0063622943,
0.0065721418,
0.0067853713,
0.0070019816,
0.0072219702,
0.0074453368,
0.0076720794,
0.0079021957,
0.0081356857,
0.0083725462,
0.0086127762,
0.0088563748,
0.0091033392,
0.0093536684,
0.0096073598,
0.0098644122,
0.0101248249,
0.0103885951,
0.0106557198,
0.0109262001,
0.0112000313,
0.0114772124,
0.0117577426,
0.0120416190,
0.0123288399,
0.0126194032,
0.0129133062,
0.0132105490,
0.0135111269,
0.0138150398,
0.0141222849,
0.0144328596,
0.0147467628,
0.0150639927,
0.0153845455,
0.0157084186,
0.0160356127,
0.0163661242,
0.0166999493,
0.0170370862,
0.0173775349,
0.0177212916,
0.0180683527,
0.0184187163,
0.0187723823,
0.0191293452,
0.0194896050,
0.0198531579,
0.0202200003,
0.0205901321,
0.0209635496,
0.0213402510,
0.0217202324,
0.0221034922,
0.0224900283,
0.0228798352,
0.0232729129,
0.0236692596,
0.0240688697,
0.0244717412,
0.0248778742,
0.0252872612,
0.0256999023,
0.0261157956,
0.0265349355,
0.0269573200,
0.0273829475,
0.0278118141,
0.0282439180,
0.0286792535,
0.0291178208,
0.0295596160,
0.0300046336,
0.0304528736,
0.0309043322,
0.0313590057,
0.0318168923,
0.0322779864,
0.0327422842,
0.0332097858,
0.0336804874,
0.0341543853,
0.0346314758,
0.0351117589,
0.0355952233,
0.0360818729,
0.0365717039,
0.0370647088,
0.0375608876,
0.0380602330,
0.0385627486,
0.0390684232,
0.0395772606,
0.0400892496,
0.0406043939,
0.0411226861,
0.0416441262,
0.0421687029,
0.0426964201,
0.0432272702,
0.0437612534,
0.0442983620,
0.0448385924,
0.0453819446,
0.0459284149,
0.0464779921,
0.0470306799,
0.0475864746,
0.0481453687,
0.0487073585,
0.0492724404,
0.0498406142,
0.0504118726,
0.0509862117,
0.0515636280,
0.0521441214,
0.0527276807,
0.0533143058,
0.0539039932,
0.0544967391,
0.0550925359,
0.0556913838,
0.0562932752,
0.0568982102,
0.0575061813,
0.0581171848,
0.0587312169,
0.0593482740,
0.0599683523,
0.0605914444,
0.0612175465,
0.0618466586,
0.0624787733,
0.0631138906,
0.0637519956,
0.0643930957,
0.0650371760,
0.0656842440,
0.0663342848,
0.0669872984,
0.0676432773,
0.0683022216,
0.0689641312,
0.0696289837,
0.0702967942,
0.0709675476,
0.0716412440,
0.0723178685,
0.0729974285,
0.0736799166,
0.0743653253,
0.0750536546,
0.0757448897,
0.0764390379,
0.0771360919,
0.0778360367,
0.0785388798,
0.0792446062,
0.0799532235,
0.0806647167,
0.0813790858,
0.0820963159,
0.0828164220,
0.0835393816,
0.0842651948,
0.0849938542,
0.0857253671,
0.0864597112,
0.0871968940,
0.0879369080,
0.0886797383,
0.0894253924,
0.0901738629,
0.0909251422,
0.0916792229,
0.0924361050,
0.0931957737,
0.0939582363,
0.0947234780,
0.0954915062,
0.0962622985,
0.0970358625,
0.0978121832,
0.0985912606,
0.0993730947,
0.1001576707,
0.1009449884,
0.1017350405,
0.1025278270,
0.1033233330,
0.1041215584,
0.1049224958,
0.1057261378,
0.1065324917,
0.1073415354,
0.1081532687,
0.1089676917,
0.1097847968,
0.1106045768,
0.1114270166,
0.1122521311,
0.1130798981,
0.1139103174,
0.1147433817,
0.1155790836,
0.1164174229,
0.1172583923,
0.1181019843,
0.1189481989,
0.1197970137,
0.1206484437,
0.1215024740,
0.1223590970,
0.1232183054,
0.1240800992,
0.1249444634,
0.1258113980,
0.1266809106,
0.1275529712,
0.1284275800,
0.1293047518,
0.1301844567,
0.1310666949,
0.1319514513,
0.1328387409,
0.1337285489,
0.1346208602,
0.1355156898,
0.1364130080,
0.1373128146,
0.1382151097,
0.1391198933,
0.1400271356,
0.1409368515,
0.1418490261,
0.1427636594,
0.1436807364,
0.1446002573,
0.1455222219,
0.1464466155,
0.1473734230,
0.1483026445,
0.1492342800,
0.1501683295,
0.1511047632,
0.1520435959,
0.1529848129,
0.1539284140,
0.1548743844,
0.1558227092,
0.1567734033,
0.1577264518,
0.1586818397,
0.1596395671,
0.1605996341,
0.1615620106,
0.1625267267,
0.1634937376,
0.1644630730,
0.1654347032,
0.1664086133,
0.1673848182,
0.1683633029,
0.1693440676,
0.1703270972,
0.1713123769,
0.1722999215,
0.1732897013,
0.1742817163,
0.1752759814,
0.1762724519,
0.1772711575,
0.1782720685,
0.1792751849,
0.1802804917,
0.1812880039,
0.1822976917,
0.1833095700,
0.1843236089,
0.1853398085,
0.1863581687,
0.1873786747,
0.1884013265,
0.1894261092,
0.1904530227,
0.1914820671,
0.1925132126,
0.1935464740,
0.1945818365,
0.1956192851,
0.1966588199,
0.1977004409,
0.1987441331,
0.1997898817,
0.2008377016,
0.2018875629,
0.2029394656,
0.2039934099,
0.2050493807,
0.2061073780,
0.2071673870,
0.2082293928,
0.2092934102,
0.2103594095,
0.2114274055,
0.2124973685,
0.2135693133,
0.2146432102,
0.2157190740,
0.2167968750,
0.2178766280,
0.2189583182,
0.2200419158,
0.2211274505,
0.2222148776,
0.2233042270,
0.2243954688,
0.2254885882,
0.2265836000,
0.2276804894,
0.2287792265,
0.2298798412,
0.2309823036,
0.2320865989,
0.2331927419,
0.2343007028,
0.2354104966,
0.2365221083,
0.2376355082,
0.2387507111,
0.2398677170,
0.2409864962,
0.2421070486,
0.2432293743,
0.2443534583,
0.2454792857,
0.2466068715,
0.2477361858,
0.2488672286,
0.2500000000,
0.2511344850,
0.2522706687,
0.2534085512,
0.2545481324,
0.2556893826,
0.2568323016,
0.2579769194,
0.2591231763,
0.2602710724,
0.2614206076,
0.2625718117,
0.2637246251,
0.2648790479,
0.2660350800,
0.2671927512,
0.2683519721,
0.2695128024,
0.2706752121,
0.2718392015,
0.2730047405,
0.2741718590,
0.2753404975,
0.2765106857,
0.2776824236,
0.2788556516,
0.2800304294,
0.2812066972,
0.2823844552,
0.2835637033,
0.2847444415,
0.2859266698,
0.2871103585,
0.2882955074,
0.2894820869,
0.2906701267,
0.2918595970,
0.2930504978,
0.2942428291,
0.2954365611,
0.2966316640,
0.2978281975,
0.2990261018,
0.3002254069,
0.3014260530,
0.3026280701,
0.3038314283,
0.3050361574,
0.3062421978,
0.3074495792,
0.3086582720,
0.3098683059,
0.3110796213,
0.3122922182,
0.3135060966,
0.3147212863,
0.3159377277,
0.3171554208,
0.3183743954,
0.3195945919,
0.3208160400,
0.3220386803,
0.3232625723,
0.3244876862,
0.3257139623,
0.3269414604,
0.3281701505,
0.3294000030,
0.3306310475,
0.3318632245,
0.3330965638,
0.3343310654,
0.3355666697,
0.3368034363,
0.3380413055,
0.3392802775,
0.3405203521,
0.3417615294,
0.3430037796,
0.3442471027,
0.3454914987,
0.3467369676,
0.3479834795,
0.3492310345,
0.3504796028,
0.3517292142,
0.3529798388,
0.3542314768,
0.3554840982,
0.3567377329,
0.3579923213,
0.3592478931,
0.3605044484,
0.3617619574,
0.3630203903,
0.3642797768,
0.3655400872,
0.3668013215,
0.3680634797,
0.3693265319,
0.3705904782,
0.3718553185,
0.3731210232,
0.3743876219,
0.3756550550,
0.3769233525,
0.3781924844,
0.3794624805,
0.3807332814,
0.3820048869,
0.3832773268,
0.3845505416,
0.3858245611,
0.3870993555,
0.3883749545,
0.3896512687,
0.3909283876,
0.3922062218,
0.3934848011,
0.3947641253,
0.3960441649,
0.3973248899,
0.3986063600,
0.3998884857,
0.4011713266,
0.4024548531,
0.4037390053,
0.4050238431,
0.4063093364,
0.4075954854,
0.4088822305,
0.4101696312,
0.4114576280,
0.4127462506,
0.4140354395,
0.4153252542,
0.4166156352,
0.4179065824,
0.4191980958,
0.4204901457,
0.4217827618,
0.4230759144,
0.4243696034,
0.4256637692,
0.4269584715,
0.4282536805,
0.4295493960,
0.4308455586,
0.4321422279,
0.4334393442,
0.4347369075,
0.4360349178,
0.4373333752,
0.4386322796,
0.4399315715,
0.4412313104,
0.4425314367,
0.4438319504,
0.4451328516,
0.4464341104,
0.4477357566,
0.4490377605,
0.4503401220,
0.4516428113,
0.4529458582,
0.4542492032,
0.4555528462,
0.4568568170,
0.4581610858,
0.4594656229,
0.4607704580,
0.4620755613,
0.4633809030,
0.4646865129,
0.4659923613,
0.4672984481,
0.4686047435,
0.4699112475,
0.4712179899,
0.4725249112,
0.4738320112,
0.4751393199,
0.4764467776,
0.4777543843,
0.4790621698,
0.4803701043,
0.4816781580,
0.4829863310,
0.4842946231,
0.4856030345,
0.4869115353,
0.4882201254,
0.4895287752,
0.4908375442,
0.4921463430,
0.4934552014,
0.4947641194,
0.4960730374,
0.4973820150,
0.4986909926,
0.5000000000,
0.5013089776,
0.5026179552,
0.5039269328,
0.5052359104,
0.5065447688,
0.5078536868,
0.5091624856,
0.5104712248,
0.5117799044,
0.5130884647,
0.5143969655,
0.5157054067,
0.5170136690,
0.5183218718,
0.5196298957,
0.5209378004,
0.5222455859,
0.5235532522,
0.5248606801,
0.5261679888,
0.5274751186,
0.5287820101,
0.5300887227,
0.5313952565,
0.5327015519,
0.5340076685,
0.5353134871,
0.5366191268,
0.5379244685,
0.5392295718,
0.5405343771,
0.5418389440,
0.5431431532,
0.5444471240,
0.5457507968,
0.5470541716,
0.5483571887,
0.5496598482,
0.5509622097,
0.5522642136,
0.5535658598,
0.5548671484,
0.5561680794,
0.5574685931,
0.5587686896,
0.5600684285,
0.5613677502,
0.5626665950,
0.5639650822,
0.5652630925,
0.5665606856,
0.5678578019,
0.5691544414,
0.5704506040,
0.5717462897,
0.5730414987,
0.5743362308,
0.5756304264,
0.5769240856,
0.5782172084,
0.5795098543,
0.5808019042,
0.5820934176,
0.5833843946,
0.5846747756,
0.5859645605,
0.5872537494,
0.5885423422,
0.5898303986,
0.5911177397,
0.5924045444,
0.5936906338,
0.5949761271,
0.5962609649,
0.5975451469,
0.5988286734,
0.6001114845,
0.6013936400,
0.6026750803,
0.6039558649,
0.6052358747,
0.6065151691,
0.6077937484,
0.6090716124,
0.6103487015,
0.6116250753,
0.6129006147,
0.6141754389,
0.6154494286,
0.6167227030,
0.6179950833,
0.6192667484,
0.6205375195,
0.6218075156,
0.6230766177,
0.6243449450,
0.6256123781,
0.6268789768,
0.6281446815,
0.6294095516,
0.6306734681,
0.6319365501,
0.6331986785,
0.6344599128,
0.6357202530,
0.6369795799,
0.6382380724,
0.6394955516,
0.6407520771,
0.6420076489,
0.6432622671,
0.6445158720,
0.6457685232,
0.6470201612,
0.6482707858,
0.6495203972,
0.6507689953,
0.6520165205,
0.6532630324,
0.6545084715,
0.6557528973,
0.6569962502,
0.6582384706,
0.6594796777,
0.6607197523,
0.6619586945,
0.6631965637,
0.6644333005,
0.6656689644,
0.6669034362,
0.6681367755,
0.6693689823,
0.6705999970,
0.6718298197,
0.6730585098,
0.6742860079,
0.6755123138,
0.6767374277,
0.6779612899,
0.6791839600,
0.6804054379,
0.6816256046,
0.6828445792,
0.6840623021,
0.6852787137,
0.6864938736,
0.6877077818,
0.6889203787,
0.6901317239,
0.6913416982,
0.6925504208,
0.6937577724,
0.6949638724,
0.6961685419,
0.6973719001,
0.6985739470,
0.6997746229,
0.7009738684,
0.7021718025,
0.7033683062,
0.7045634389,
0.7057572007,
0.7069494724,
0.7081403732,
0.7093298435,
0.7105178833,
0.7117044926,
0.7128896713,
0.7140733600,
0.7152555585,
0.7164362669,
0.7176155448,
0.7187933326,
0.7199695706,
0.7211443186,
0.7223175764,
0.7234892845,
0.7246595025,
0.7258281708,
0.7269952297,
0.7281607985,
0.7293247581,
0.7304871678,
0.7316480279,
0.7328072786,
0.7339649200,
0.7351209521,
0.7362753749,
0.7374281883,
0.7385793924,
0.7397289276,
0.7408768535,
0.7420231104,
0.7431676984,
0.7443106174,
0.7454518676,
0.7465914488,
0.7477293611,
0.7488655448,
0.7500000000,
0.7511327863,
0.7522637844,
0.7533931136,
0.7545207143,
0.7556465268,
0.7567706108,
0.7578929663,
0.7590135336,
0.7601323128,
0.7612493038,
0.7623645067,
0.7634779215,
0.7645894885,
0.7656992674,
0.7668072581,
0.7679134011,
0.7690176964,
0.7701201439,
0.7712207437,
0.7723194957,
0.7734164000,
0.7745113969,
0.7756045461,
0.7766957879,
0.7777851224,
0.7788725495,
0.7799580693,
0.7810416818,
0.7821233869,
0.7832031250,
0.7842808962,
0.7853567600,
0.7864306569,
0.7875026464,
0.7885726094,
0.7896406054,
0.7907065749,
0.7917705774,
0.7928326130,
0.7938926220,
0.7949506044,
0.7960065603,
0.7970605493,
0.7981124520,
0.7991623282,
0.8002101183,
0.8012558818,
0.8022995591,
0.8033411503,
0.8043807149,
0.8054181933,
0.8064535260,
0.8074867725,
0.8085179329,
0.8095469475,
0.8105738759,
0.8115986586,
0.8126213551,
0.8136418462,
0.8146601915,
0.8156763911,
0.8166904449,
0.8177022934,
0.8187119961,
0.8197194934,
0.8207248449,
0.8217279315,
0.8227288723,
0.8237275481,
0.8247240186,
0.8257182837,
0.8267102838,
0.8277000785,
0.8286876082,
0.8296729326,
0.8306559324,
0.8316366673,
0.8326151967,
0.8335914016,
0.8345652819,
0.8355369568,
0.8365062475,
0.8374732733,
0.8384379745,
0.8394003510,
0.8403604627,
0.8413181901,
0.8422735333,
0.8432266116,
0.8441773057,
0.8451256156,
0.8460716009,
0.8470152020,
0.8479564190,
0.8488952518,
0.8498316407,
0.8507657051,
0.8516973257,
0.8526265621,
0.8535534143,
0.8544777632,
0.8553997278,
0.8563192487,
0.8572363257,
0.8581509590,
0.8590631485,
0.8599728942,
0.8608801365,
0.8617848754,
0.8626871705,
0.8635870218,
0.8644843102,
0.8653791547,
0.8662714362,
0.8671612740,
0.8680485487,
0.8689333200,
0.8698155284,
0.8706952333,
0.8715724349,
0.8724470139,
0.8733190894,
0.8741886020,
0.8750555515,
0.8759198785,
0.8767817020,
0.8776409030,
0.8784975410,
0.8793515563,
0.8802030087,
0.8810517788,
0.8818979859,
0.8827416301,
0.8835825920,
0.8844209313,
0.8852566481,
0.8860896826,
0.8869200945,
0.8877478838,
0.8885729909,
0.8893954158,
0.8902152181,
0.8910322785,
0.8918467164,
0.8926584721,
0.8934674859,
0.8942738771,
0.8950775266,
0.8958784342,
0.8966766596,
0.8974722028,
0.8982649446,
0.8990550041,
0.8998423219,
0.9006268978,
0.9014087319,
0.9021878242,
0.9029641151,
0.9037377238,
0.9045084715,
0.9052765369,
0.9060417414,
0.9068042040,
0.9075639248,
0.9083207846,
0.9090748429,
0.9098261595,
0.9105746150,
0.9113202691,
0.9120631218,
0.9128031135,
0.9135403037,
0.9142746329,
0.9150061607,
0.9157348275,
0.9164606333,
0.9171835780,
0.9179036617,
0.9186209440,
0.9193353057,
0.9200468063,
0.9207553864,
0.9214611053,
0.9221639633,
0.9228639007,
0.9235609770,
0.9242551327,
0.9249463677,
0.9256346822,
0.9263200760,
0.9270025492,
0.9276821017,
0.9283587337,
0.9290324450,
0.9297031760,
0.9303709865,
0.9310358763,
0.9316977859,
0.9323567152,
0.9330127239,
0.9336656928,
0.9343157411,
0.9349628091,
0.9356068969,
0.9362480044,
0.9368861318,
0.9375212193,
0.9381533265,
0.9387824535,
0.9394085407,
0.9400316477,
0.9406517148,
0.9412688017,
0.9418827891,
0.9424937963,
0.9431017637,
0.9437067509,
0.9443086386,
0.9449074864,
0.9455032349,
0.9460960031,
0.9466856718,
0.9472723007,
0.9478558898,
0.9484363794,
0.9490137696,
0.9495881200,
0.9501593709,
0.9507275820,
0.9512926340,
0.9518546462,
0.9524134994,
0.9529693127,
0.9535220265,
0.9540715814,
0.9546180367,
0.9551613927,
0.9557016492,
0.9562387466,
0.9567727447,
0.9573035836,
0.9578313231,
0.9583559036,
0.9588773251,
0.9593955874,
0.9599107504,
0.9604227543,
0.9609315991,
0.9614372253,
0.9619397521,
0.9624391198,
0.9629352689,
0.9634283185,
0.9639181495,
0.9644047618,
0.9648882151,
0.9653685093,
0.9658455849,
0.9663195014,
0.9667901993,
0.9672577381,
0.9677219987,
0.9681831002,
0.9686409831,
0.9690956473,
0.9695471525,
0.9699953794,
0.9704403877,
0.9708821774,
0.9713207483,
0.9717561007,
0.9721881747,
0.9726170301,
0.9730426669,
0.9734650850,
0.9738842249,
0.9743000865,
0.9747127295,
0.9751221538,
0.9755282402,
0.9759311080,
0.9763307571,
0.9767270684,
0.9771201611,
0.9775099754,
0.9778965116,
0.9782797694,
0.9786597490,
0.9790364504,
0.9794098735,
0.9797800183,
0.9801468253,
0.9805104136,
0.9808706641,
0.9812276363,
0.9815812707,
0.9819316268,
0.9822787046,
0.9826224446,
0.9829629064,
0.9833000302,
0.9836338758,
0.9839643836,
0.9842915535,
0.9846154451,
0.9849359989,
0.9852532148,
0.9855671525,
0.9858776927,
0.9861849546,
0.9864888787,
0.9867894650,
0.9870867133,
0.9873806238,
0.9876711369,
0.9879583716,
0.9882422686,
0.9885227680,
0.9887999892,
0.9890738130,
0.9893442988,
0.9896113873,
0.9898751974,
0.9901356101,
0.9903926253,
0.9906463027,
0.9908966422,
0.9911436439,
0.9913872480,
0.9916274548,
0.9918643236,
0.9920977950,
0.9923279285,
0.9925546646,
0.9927780032,
0.9929980040,
0.9932146072,
0.9934278727,
0.9936376810,
0.9938441515,
0.9940472245,
0.9942469597,
0.9944432378,
0.9946361780,
0.9948257208,
0.9950118065,
0.9951945543,
0.9953739047,
0.9955498576,
0.9957224131,
0.9958915710,
0.9960573316,
0.9962196946,
0.9963786602,
0.9965342283,
0.9966863990,
0.9968351126,
0.9969804883,
0.9971224070,
0.9972609282,
0.9973960519,
0.9975277781,
0.9976561069,
0.9977809787,
0.9979024529,
0.9980205297,
0.9981352091,
0.9982464314,
0.9983542562,
0.9984586835,
0.9985596538,
0.9986572266,
0.9987514019,
0.9988421202,
0.9989294410,
0.9990133643,
0.9990938306,
0.9991708994,
0.9992445707,
0.9993147850,
0.9993815422,
0.9994449615,
0.9995048642,
0.9995614290,
0.9996145368,
0.9996641874,
0.9997104406,
0.9997532964,
0.9997926950,
0.9998286366,
0.9998612404,
0.9998903275,
0.9999160171,
0.9999383092,
0.9999571443,
0.9999725819,
0.9999845624,
0.9999931455,
0.9999982715,
];
