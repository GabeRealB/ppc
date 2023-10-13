/* eslint no-magic-numbers: 0 */
import React, { Component } from 'react';

import PPC from '../components/PPC';

class App extends Component {

    constructor(props) {
        super(props);
        this.state = {
            axes: {
                "Var 1": {
                    range: [0, 100],
                    values: [...Array(100)].map(() => Math.random() * 100)
                },
                "Var 2": {
                    range: [15, 30],
                    values: [...Array(100)].map(() => Math.random() * (30 - 15) + 15)
                },
                "Var 3": {
                    range: [0, 1],
                    values: [...Array(100)].map(() => (Math.random()) > 0.5 ? 1 : 0)
                },
                "Var 4": {
                    range: [15, 30],
                    values: [...Array(100)].map(() => Math.random() * (30 - 15) + 15)
                },
                "Var 5": {
                    range: [15, 30],
                    values: [...Array(100)].map(() => Math.random() * (30 - 15) + 15)
                },
            }
        };
        this.setProps = this.setProps.bind(this);
    }

    setProps(newProps) {
        this.setState(newProps);
    }

    render() {
        return (
            <div>
                <PPC
                    setProps={this.setProps}
                    {...this.state}
                />
            </div>
        )
    }
}

export default App;
