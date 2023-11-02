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
                    // visibleRange: [20, 70],
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
                "v_6": {
                    label: "Var 6",
                    range: [15, 30],
                    datums: [...Array(100)].map(() => Math.random() * (30 - 15) + 15),
                    hidden: true
                },
            },
            labels: {
                "label_1": {},
                "label_2": {}
            },
            active_label: "label_1",
            demo_slider_value: Number.EPSILON
        };
        this.setProps = this.setProps.bind(this);
    }

    setProps(newProps) {
        this.setState(newProps);
    }

    render() {
        return (
            <div style={{ display: 'flex', width: "100%", height: "100%" }} >
                <div style={{ width: "85%", height: "100%", paddingLeft: "2rem", paddingRight: "2rem" }}>
                    <PPC
                        setProps={this.setProps}
                        {...this.state}
                    />
                </div>
                <div style={{ flexDirection: "column" }}>
                    <h2>Labels</h2>
                    <div style={{ display: 'flex', flexDirection: "row" }}>
                        <label style={{ paddingRight: "1rem" }}>Label</label>
                        <select onChange={e => {
                            const active_label = e.target.value;
                            const demo_slider_value = "selection_threshold" in this.state["labels"][active_label] ? this.state["labels"][active_label]["selection_threshold"] : Number.EPSILON;

                            this.setProps({ active_label: active_label, demo_slider_value: demo_slider_value })
                        }}>
                            <option value="label_1">Label 1</option>
                            <option value="label_2">Label 2</option>
                        </select>
                    </div>
                    <div style={{ display: 'flex', flexDirection: "row" }}>
                        <label style={{ paddingRight: "1rem" }}>Selection threshold</label>
                        <input type="range" min={0.0} max={1.0} step={Number.EPSILON} value={this.state["demo_slider_value"]} onChange={e => {
                            const labels = window.structuredClone(this.state["labels"]);
                            const active_label = this.state["active_label"];
                            labels[active_label]["selection_threshold"] = parseFloat(e.target.value);
                            this.setProps({ labels, demo_slider_value: parseFloat(e.target.value) })
                        }}></input>
                    </div>
                    <h2>Color scale</h2>
                    <h2>Color bar</h2>
                </div>
            </div>
        )
    }
}

export default App;
