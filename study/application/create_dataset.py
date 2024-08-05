import os
import json
import random

num_points = 2000

a1_range = [0, 50]
a2_range = [100, 200]

def clamp(x, min_v, max_v):
    return max(min_v, min(max_v, x))

def create_a1_value():
    return random.uniform(*a1_range)

def create_a2_value():
    return random.uniform(*a2_range)

def ease_constant(x: float, start: float, end: float):
    if start <= x <= end:
        return 1
    else:
        return 0

def ease_linear(x: float, min_v: float, max_v: float):
    x = (x - min(min_v, max_v)) / abs(max_v - min_v)
    if x < 0 or x > 1:
        return 0
    
    if min_v <= max_v:
        return x
    else:
        return 1 - x

def ease_in_out(x: float, min_v: float, max_v: float):
    x = (x - min(min_v, max_v)) / abs(max_v - min_v)
    if x < 0 or x > 1:
        return 0

    if min_v > max_v:
        x = 1 - x

    if x < 0.5:
        return 4 * (x ** 3)
    else:
        return 1 - ((-2 * x + 2) ** 3) / 2

def sample_a1_curve(x: float):
    v = 0
    v = max(v, ease_in_out(x, 10, 22.5))
    v = max(v, ease_constant(x, 22.5, 27.5))
    v = max(v, ease_in_out(x, 40, 27.5))
    return v

def sample_a2_curve(x: float):
    v = 0
    v = max(v, ease_in_out(x, 100, 120))
    v = max(v, ease_constant(x, 120, 130))
    v = max(v, ease_in_out(x, 200, 130))
    return v

def compute_selection_probability(a1: float, a2: float):
    p1 = sample_a1_curve(a1)
    p2 = sample_a2_curve(a2)

    return p1 * p2

def compute_class(a1: float, a2: float):
    prob = compute_selection_probability(a1, a2)

    if 0 < prob <= 0.25:
        return 0.75
    else:
        return 0.25

def create_dataset():
    labels = [i for i in range(num_points)]
    a1 = [create_a1_value() for _ in range(num_points)]
    a2 = [create_a2_value() for _ in range(num_points)]
    cl = [compute_class(a1, a2) for a1, a2 in zip(a1, a2)]

    return {
        'a1': a1,
        'a2': a2,
        'labels': labels,
        'cl': cl,
    }

def export_json(dataset):
    obj = {
        'a1': {
            'label': 'A1',
            'range': a1_range,
            'dataPoints': dataset['a1']
        },
        'a2': {
            'label': 'A2',
            'range': a2_range,
            'dataPoints': dataset['a2']
        },
        'label': {
            'label': 'Label',
            'dataPoints': dataset['labels']
        },
        'class': {
            'label': 'Class',
            'range': [0, 1],
            'tickPositions': [0.25, 0.75],
            'tickLabels': ['Not selected', 'Selected'],
            'dataPoints': dataset['cl']
        }
    }

    file_path = os.path.join(os.path.dirname(os.path.realpath(__file__)), 'output.json')
    with open(file_path, 'w') as f:
        json.dump(obj, f, indent=4)

if __name__ == '__main__':
    dataset = create_dataset()
    export_json(dataset)