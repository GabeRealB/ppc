import ppc
import dash

app = dash.Dash(__name__)

app.layout = ppc.PPC(id='component')


if __name__ == '__main__':
    app.run_server(debug=True)
