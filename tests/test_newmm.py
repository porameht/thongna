import unittest
from typing import List

from thongna import load_dict, newmm

class TestTokenizePackage(unittest.TestCase):
    def setUp(self):
        self.TEXT_1 = "สวัสดีชาวโลกHello World 123 !@#$%^&*"
        self.TEXT_2 = "การทดสอบ"
        self.LONG_TEXT = "ไต้หวัน (แป่ะเอ๋ยี้: Tâi-oân; ไต่อวัน) หรือ ไถวาน (อักษรโรมัน: Taiwan; จีนตัวย่อ: 台湾; จีนตัวเต็ม: 臺灣/台灣; พินอิน: Táiwān; ไถวาน) หรือชื่อทางการว่า สาธารณรัฐจีน (จีนตัวย่อ: 中华民国; จีนตัวเต็ม: 中華民國; พินอิน: Zhōnghuá Mínguó) เป็นรัฐในทวีปเอเชียตะวันออก"
        self.DANGER_TEXT_1 = "ชิ" * 100
        self.DANGER_TEXT_2 = "ด้านหน้า" * 20
        self.DANGER_TEXT_3 = "ด้านหน้า" * 20 + "ก" * 40

        self.DICT_FILENAME = "dataset/words_th.txt"
        self.DICT_NAME = "words_th"
        load_dict(self.DICT_FILENAME, self.DICT_NAME)

    def test_segment_empty_input(self):
        self.assertEqual(newmm(None, self.DICT_NAME), [])
        self.assertEqual(newmm("", self.DICT_NAME), [])
        self.assertEqual(newmm(" ", self.DICT_NAME), [" "])

    def test_segment_simple_input(self):
        self.assertEqual(
            newmm("ไข่คน2021", self.DICT_NAME),
            ["ไข่", "คน", "2021"],
        )

    def test_segment_with_dict_words(self):
        result = newmm(
            "ค่าจ้างที่ได้รับต้องทำให้แรงงาน"
            "สามารถเลี้ยงดูตัวเองและครอบครัว"
            "อย่างสมศักดิ์ศรีความเป็นมนุษย์",
            self.DICT_NAME,
        )
        self.assertIn("ค่าจ้าง", result)
        self.assertIn("แรงงาน", result)
        self.assertIn("ครอบครัว", result)

    def test_segment_various_inputs(self):
        inputs = [
            self.TEXT_1,
            self.TEXT_2,
            self.LONG_TEXT,
            self.DANGER_TEXT_1,
            self.DANGER_TEXT_2,
            self.DANGER_TEXT_3
        ]
        for text in inputs:
            with self.subTest(text=text):
                result = newmm(text, self.DICT_NAME)
                self.assertIsInstance(result, List)
                self.assertGreater(len(result), 0)

    def test_segment_with_numbers_and_special_chars(self):
        result = newmm("ราคา ฿550.75 บาท", self.DICT_NAME)
        self.assertIn("ราคา", result)
        self.assertIn("฿", result)
        self.assertIn("550.75", result)
        self.assertIn("บาท", result)

    def test_segment_with_english_words(self):
        result = newmm("เขาชอบ pizza มาก", self.DICT_NAME)
        self.assertIn("เขา", result)
        self.assertIn("ชอบ", result)
        self.assertIn("pizza", result)
        self.assertIn("มาก", result)

    def test_segment_with_misspelled_words(self):
        result = newmm("เคาไปโรงเรียน", self.DICT_NAME)
        self.assertIn("เคา", result)
        self.assertIn("ไป", result)
        self.assertIn("โรงเรียน", result)

    def test_segment_with_long_word(self):
        long_word = "ยาว" * 50
        result = newmm(long_word, self.DICT_NAME)
        self.assertGreater(len(result), 1)  # Should be split into multiple tokens

    def test_segment_with_parallel_mode(self):
        result_parallel = newmm(self.LONG_TEXT, self.DICT_NAME, parallel=True)
        result_sequential = newmm(self.LONG_TEXT, self.DICT_NAME, parallel=False)
        self.assertEqual(result_parallel, result_sequential)