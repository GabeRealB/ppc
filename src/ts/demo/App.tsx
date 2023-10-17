/* eslint no-magic-numbers: 0 */
import React, { Component } from 'react';

import PPC from '../components/PPC';

class App extends Component {

    constructor(props) {
        super(props);
        this.state = {
            axes: {
                "v_1": {
                    label: "Var 1",
                    range: [0, 100],
                    datums: [...Array(100)].map(() => Math.random() * 100)
                },
                "v_2": {
                    label: "Var 2",
                    range: [15, 30],
                    datums: [...Array(100)].map(() => Math.random() * (30 - 15) + 15)
                },
                "v_3": {
                    label: "Var 3",
                    range: [0, 1],
                    datums: [...Array(100)].map(() => (Math.random()) > 0.5 ? 1 : 0)
                },
                "v_4": {
                    label: "Var 4",
                    range: [15, 30],
                    datums: [...Array(100)].map(() => Math.random() * (30 - 15) + 15)
                },
                "v_5": {
                    label: "Var 5",
                    range: [15, 30],
                    datums: [...Array(100)].map(() => Math.random() * (30 - 15) + 15)
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
