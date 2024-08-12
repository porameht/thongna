import thongna

text = "สวัสดีครับ ผมชื่อไท"
tokens = thongna.tokenize(text)
print(tokens)  # Output: ['สวัสดีครับ', 'ผมชื่อไท']

reversed_text = thongna.reverse_text(text)
print(reversed_text)  # Output: 'ทิชูมผ บครีดัสวัส'


text = "สระะน้ำ"
whitespace_number = False

# เรียกใช้ฟังก์ชัน normalize
normalized_text = thongna.normalize(text, whitespace_number)
print(f"Original text: {text}")
print(f"Normalized text: {normalized_text}")

# ทดสอบกรณีที่ whitespace_number เป็น True
text_with_numbers = "123 สระะน้ำ 456"
normalized_text_with_numbers = thongna.normalize(text_with_numbers, True)
print(f"Original text with numbers: {text_with_numbers}")
print(f"Normalized text with numbers: {normalized_text_with_numbers}")