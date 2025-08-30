import pytest
from core.text.processing import clean_and_lemmatize

@pytest.mark.parametrize(
    "input_text,expected_output",
    [
        ("World is Burning! Stop climate change", ['world', 'burn', 'stop', 'climate', 'change']),
        ("Hello, world! This is a test.", ['hello', 'world', 'test']),
        ("the and is of", []),
        ("running leaves cars", ['run', 'leave', 'car']),
        ("Python 3.9 rocks!", ['python', '3.9', 'rock']),
        ("", []),
    ]
)
def test_clean_and_lemmatize(input_text, expected_output):
    result = clean_and_lemmatize(input_text)
    assert result == expected_output