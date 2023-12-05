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
            colors: {
                selected: {
                    scale: "viridis",
                    color: 0.5,
                }
            },
            colorBar: "visible",
            labels: {
                "label_1": {},
                "label_2": {},
                "label_3": {}
            },
            activeLabel: "label_1",
            debug: {
                showAxisBoundingBox: false,
                showLabelBoundingBox: false,
                showCurvesBoundingBox: false,
                showAxisLineBoundingBox: false,
                showSelectionsBoundingBox: false,
                showColorBarBoundingBox: false,
            },
            demo_slider_value: Number.EPSILON,
            coloring_constant_value: 0.5,
            coloring_attribute: "v_1",
        };
        this.setProps = this.setProps.bind(this);
    }

    setProps(newProps) {
        this.setState(newProps);
    }

    render() {
        return (
            <div style={{ display: "flex", width: "100%", height: "100%", alignItems: "flex-start" }} >
                <div style={{ width: "85%", height: "100%", paddingLeft: "2rem", paddingRight: "2rem" }}>
                    <PPC
                        setProps={this.setProps}
                        {...this.state}
                    />
                </div>
                <div style={{ display: "grid", gridTemplateColumns: "repeat(1, 1fr)", gap: "10px", gridAutoRows: "minmax(20px, auto)", alignItems: "flex-start", overflow: "auto" }}>
                    <h2 style={{ gridRow: 1 }}>Labels</h2>

                    <label style={{ gridRow: 2, gridColumn: "1" }}>Label</label>
                    <select style={{ gridRow: 2, gridColumn: "2" }} onChange={e => {
                        const activeLabel = e.target.value;
                        const demo_slider_value = "selection_threshold" in this.state["labels"][activeLabel] ? this.state["labels"][activeLabel]["selection_threshold"] : Number.EPSILON;

                        this.setProps({ activeLabel: activeLabel, demo_slider_value: demo_slider_value })
                    }}>
                        <option value="label_1">Label 1</option>
                        <option value="label_2">Label 2</option>
                        <option value="label_3">Label 3</option>
                    </select>

                    <label style={{ gridRow: 3, gridColumn: "1" }}>Selection threshold</label>
                    <input style={{ gridRow: 3, gridColumn: "2" }} type="range" min={0.0} max={1.0} step={Number.EPSILON} value={this.state["demo_slider_value"]} onChange={e => {
                        const labels = window.structuredClone(this.state["labels"]);
                        const activeLabel = this.state["activeLabel"];
                        labels[activeLabel]["selection_threshold"] = parseFloat(e.target.value);
                        this.setProps({ labels, demo_slider_value: parseFloat(e.target.value) })
                    }}></input>

                    <h2 style={{ gridRow: 4, gridColumn: "1 / 2" }}>Color scale</h2>

                    <label style={{ gridRow: 5, gridColumn: "1" }}>Show color scale</label>
                    <input style={{ gridRow: 5, gridColumn: "2" }} type="checkbox" checked={this.state["colorBar"] === "visible"} onChange={e => {
                        const colorBar = this.state["colorBar"];
                        if (colorBar === "hidden") {
                            this.setProps({ colorBar: "visible" })
                        } else {
                            this.setProps({ colorBar: "hidden" })
                        }
                    }}></input>


                    <label style={{ gridRow: 6, gridColumn: "1" }}>Datums coloring</label>
                    <label style={{ gridRow: 6, gridColumn: "2" }}><input type="radio" name="coloring" value="constant" checked={typeof (this.state["colors"].selected.color) === "number"} onChange={e => {
                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.color = this.state["coloring_constant_value"];
                        this.setProps({ colors })
                    }} />Constant</label>
                    <label style={{ gridRow: 7, gridColumn: "2" }}><input type="radio" name="coloring" value="attribute" checked={typeof (this.state["colors"].selected.color) === "string"} onChange={e => {
                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.color = this.state["coloring_attribute"];
                        this.setProps({ colors })
                    }} />Attribute</label>
                    <label style={{ gridRow: 8, gridColumn: "2" }}><input type="radio" name="coloring" value="probability" checked={this.state["colors"].selected.color.type === "probability"} onChange={e => {
                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.color = { type: "probability" };
                        this.setProps({ colors })
                    }} />Probability</label>


                    <label style={{ gridRow: 9, gridColumn: "1" }}>Coloring constant</label>
                    <input style={{ gridRow: 9, gridColumn: "2" }} type="range" min={0.0} max={1.0} step={Number.EPSILON} value={this.state["coloring_constant_value"]} onChange={e => {
                        const coloring_constant_value = parseFloat(e.target.value);
                        if (typeof (this.state["colors"].selected.color) !== "number") {
                            this.setProps({ coloring_constant_value })
                            return;
                        }

                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.color = coloring_constant_value;
                        this.setProps({ colors, coloring_constant_value })
                    }}></input>


                    <label style={{ gridRow: 10, gridColumn: "1" }}>Coloring attribute</label>
                    <label style={{ gridRow: 10, gridColumn: "2" }}><input type="radio" name="attribute" value="v_1" checked={this.state["coloring_attribute"] === "v_1"} onChange={e => {
                        const coloring_attribute = e.target.value;
                        if (typeof (this.state["colors"].selected.color) !== "string") {
                            this.setProps({ coloring_attribute })
                            return;
                        }

                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.color = coloring_attribute;
                        this.setProps({ colors, coloring_attribute })
                    }} />Var 1</label>
                    <label style={{ gridRow: 11, gridColumn: "2" }}><input type="radio" name="attribute" value="v_2" checked={this.state["coloring_attribute"] === "v_2"} onChange={e => {
                        const coloring_attribute = e.target.value;
                        if (typeof (this.state["colors"].selected.color) !== "string") {
                            this.setProps({ coloring_attribute })
                            return;
                        }

                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.color = coloring_attribute;
                        this.setProps({ colors, coloring_attribute })
                    }} />Var 2</label>
                    <label style={{ gridRow: 12, gridColumn: "2" }}><input type="radio" name="attribute" value="v_3" checked={this.state["coloring_attribute"] === "v_3"} onChange={e => {
                        const coloring_attribute = e.target.value;
                        if (typeof (this.state["colors"].selected.color) !== "string") {
                            this.setProps({ coloring_attribute })
                            return;
                        }

                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.color = coloring_attribute;
                        this.setProps({ colors, coloring_attribute })
                    }} />Var 3</label>
                    <label style={{ gridRow: 13, gridColumn: "2" }}><input type="radio" name="attribute" value="v_4" checked={this.state["coloring_attribute"] === "v_4"} onChange={e => {
                        const coloring_attribute = e.target.value;
                        if (typeof (this.state["colors"].selected.color) !== "string") {
                            this.setProps({ coloring_attribute })
                            return;
                        }

                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.color = coloring_attribute;
                        this.setProps({ colors, coloring_attribute })
                    }} />Var 4</label>
                    <label style={{ gridRow: 14, gridColumn: "2" }}><input type="radio" name="attribute" value="v_5" checked={this.state["coloring_attribute"] === "v_5"} onChange={e => {
                        const coloring_attribute = e.target.value;
                        if (typeof (this.state["colors"].selected.color) !== "string") {
                            this.setProps({ coloring_attribute })
                            return;
                        }

                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.color = coloring_attribute;
                        this.setProps({ colors, coloring_attribute })
                    }} />Var 5</label>
                    <label style={{ gridRow: 15, gridColumn: "2" }}><input type="radio" name="attribute" value="v_6" checked={this.state["coloring_attribute"] === "v_6"} onChange={e => {
                        const coloring_attribute = e.target.value;
                        if (typeof (this.state["colors"].selected.color) !== "string") {
                            this.setProps({ coloring_attribute })
                            return;
                        }

                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.color = coloring_attribute;
                        this.setProps({ colors, coloring_attribute })
                    }} />Var 6</label>


                    <label style={{ gridRow: 16, gridColumn: "1" }}>Color map</label>
                    <label style={{ gridRow: 16, gridColumn: "2" }}><input type="radio" name="color_map" value="magma" checked={this.state["colors"].selected.scale === "magma"} onChange={e => {
                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.scale = e.target.value;
                        this.setProps({ colors })
                    }} />Magma</label>
                    <label style={{ gridRow: 17, gridColumn: "2" }}><input type="radio" name="color_map" value="inferno" checked={this.state["colors"].selected.scale === "inferno"} onChange={e => {
                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.scale = e.target.value;
                        this.setProps({ colors })
                    }} />Inferno</label>
                    <label style={{ gridRow: 18, gridColumn: "2" }}><input type="radio" name="color_map" value="plasma" checked={this.state["colors"].selected.scale === "plasma"} onChange={e => {
                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.scale = e.target.value;
                        this.setProps({ colors })
                    }} />Plasma</label>
                    <label style={{ gridRow: 19, gridColumn: "2" }}><input type="radio" name="color_map" value="viridis" checked={this.state["colors"].selected.scale === "viridis"} onChange={e => {
                        const colors = window.structuredClone(this.state["colors"]);
                        colors.selected.scale = e.target.value;
                        this.setProps({ colors })
                    }} />Viridis</label>


                    <h2 style={{ gridRow: 20 }}>Debug</h2>

                    <h3 style={{ gridRow: 21 }}>Bounding Boxes</h3>
                    <label style={{ gridRow: 22, gridColumn: "1" }}>Axis</label>
                    <input style={{ gridRow: 22, gridColumn: "2" }} type="checkbox" checked={this.state["debug"].showAxisBoundingBox} onChange={e => {
                        const debug = window.structuredClone(this.state["debug"]);
                        debug.showAxisBoundingBox = !debug.showAxisBoundingBox;
                        this.setProps({ debug })
                    }}></input>
                    <label style={{ gridRow: 23, gridColumn: "1" }}>Label</label>
                    <input style={{ gridRow: 23, gridColumn: "2" }} type="checkbox" checked={this.state["debug"].showLabelBoundingBox} onChange={e => {
                        const debug = window.structuredClone(this.state["debug"]);
                        debug.showLabelBoundingBox = !debug.showLabelBoundingBox;
                        this.setProps({ debug })
                    }}></input>
                    <label style={{ gridRow: 24, gridColumn: "1" }}>Curves</label>
                    <input style={{ gridRow: 24, gridColumn: "2" }} type="checkbox" checked={this.state["debug"].showCurvesBoundingBox} onChange={e => {
                        const debug = window.structuredClone(this.state["debug"]);
                        debug.showCurvesBoundingBox = !debug.showCurvesBoundingBox;
                        this.setProps({ debug })
                    }}></input>
                    <label style={{ gridRow: 25, gridColumn: "1" }}>Axis line</label>
                    <input style={{ gridRow: 25, gridColumn: "2" }} type="checkbox" checked={this.state["debug"].showAxisLineBoundingBox} onChange={e => {
                        const debug = window.structuredClone(this.state["debug"]);
                        debug.showAxisLineBoundingBox = !debug.showAxisLineBoundingBox;
                        this.setProps({ debug })
                    }}></input>
                    <label style={{ gridRow: 26, gridColumn: "1" }}>Selections</label>
                    <input style={{ gridRow: 26, gridColumn: "2" }} type="checkbox" checked={this.state["debug"].showSelectionsBoundingBox} onChange={e => {
                        const debug = window.structuredClone(this.state["debug"]);
                        debug.showSelectionsBoundingBox = !debug.showSelectionsBoundingBox;
                        this.setProps({ debug })
                    }}></input>
                    <label style={{ gridRow: 27, gridColumn: "1" }}>Colorbar</label>
                    <input style={{ gridRow: 27, gridColumn: "2" }} type="checkbox" checked={this.state["debug"].showColorBarBoundingBox} onChange={e => {
                        const debug = window.structuredClone(this.state["debug"]);
                        debug.showColorBarBoundingBox = !debug.showColorBarBoundingBox;
                        this.setProps({ debug })
                    }}></input>
                </div>
            </div>
        )
    }
}

export default App;
