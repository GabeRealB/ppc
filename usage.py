import ppc
import random
import dash
from dash import html

LINE_COUNT = 100

app = dash.Dash(__name__, assets_folder='deps')

props = {
    "axes": {
        "a1": {
            "label": "A1",
            "range": [0, 10],
            "dataPoints": [random.random() * 10 for _ in range(LINE_COUNT)]
        },
        "a2": {
            "label": "A2",
            "range": [0, 10],
            "dataPoints": [random.random() * 10 for _ in range(LINE_COUNT)]
        },
        "a3": {
            "label": "A3",
            "range": [0, 10],
            "dataPoints": [random.random() * 10 for _ in range(LINE_COUNT)]
        }
    },
    "order": ["a1", "a2", "a3"],
    "labels": {
        "Default": {},
    },
    "activeLabel": "Default",
    "colors": {
        "selected": {
            "scale": "plasma",
            "color": 0.5,
        }
    },
    "colorBar": "hidden",
    "interactionMode": 2,
    "debug": {
        "showAxisBoundingBox": False,
        "showLabelBoundingBox": False,
        "showCurvesBoundingBox": False,
        "showAxisLineBoundingBox": False,
        "showSelectionsBoundingBox": False,
        "showColorBarBoundingBox": False,
    }
}

app.layout = html.Div([
    ppc.PPC(id='component', **props)
], style={
    "height": "1000px"
})

if __name__ == '__main__':
    app.run_server(debug=True)
