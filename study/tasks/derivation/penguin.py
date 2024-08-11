import os
import csv
import json

if __name__ == "__main__":
    ids = []
    species = []
    islands = []
    bill_lengths = []
    bill_depths = []
    flipper_lengths = []
    body_mass_gs = []
    sexes = []
    years = []

    file_path = os.path.join(os.path.dirname(os.path.realpath(__file__)), 'penguins.csv')
    with open(file_path, 'r') as f:
        reader = csv.reader(f)
        next(reader, None)

        for row in reader:
            ids.append(int(row[0]))
            species.append(str(row[1]))
            islands.append(str(row[2]))
            bill_lengths.append(float(row[3]))
            bill_depths.append(float(row[4]))
            flipper_lengths.append(int(row[5]))
            body_mass_gs.append(int(row[6]))
            sexes.append(str(row[7]))
            years.append(int(row[8]))

    species_map = { 'Adelie': 0.25, 'Chinstrap': 0.5, 'Gentoo': 0.75 }
    islands_map = { 'Biscoe': 0.25, 'Dream': 0.5, 'Torgersen': 0.75 }
    sex_map = { 'female': 0.25, 'male': 0.75 }

    species = list(map(lambda x: species_map[x], species))
    islands = list(map(lambda x: islands_map[x], islands))
    sexes = list(map(lambda x: sex_map[x], sexes))

    obj = {
        'id': {
            'label': 'id',
            'dataPoints': ids,
        },
        'species': {
            'label': 'species',
            'dataPoints': species,
            'range': [0, 1],
            'tickPositions': [0.25, 0.5, 0.75],
            'tickLabels': ['Adelie', 'Chinstrap', 'Gentoo']
        },
        'island': {
            'label': 'island',
            'dataPoints': islands,
            'range': [0, 1],
            'tickPositions': [0.25, 0.5, 0.75],
            'tickLabels': ['Biscoe', 'Dream', 'Torgersen']
        },
        'bill_length_mm': {
            'label': 'bill_length_mm',
            'dataPoints': bill_lengths,
        },
        'bill_depth_mm': {
            'label': 'bill_depth_mm',
            'dataPoints': bill_depths,
        },
        'flipper_length_mm': {
            'label': 'flipper_length_mm',
            'dataPoints': flipper_lengths,
        },
        'body_mass_g': {
            'label': 'body_mass_g',
            'dataPoints': body_mass_gs,
        },
        'sex': {
            'label': 'sex',
            'dataPoints': sexes,
            'range': [0, 1],
            'tickPositions': [0.25, 0.75],
            'tickLabels': ['female', 'male']
        },
        'year': {
            'label': 'year',
            'dataPoints': years,
        },
    }
    
    file_path = os.path.join(os.path.dirname(os.path.realpath(__file__)), 'penguin.json')
    with open(file_path, 'w') as f:
        json.dump(obj, f, indent=4)
