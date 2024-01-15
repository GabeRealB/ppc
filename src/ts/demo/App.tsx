/* eslint no-magic-numbers: 0 */
import React, { Component } from 'react';
import { Box, MenuItem, Select, Stack, FormControl, Typography, InputLabel, FormControlLabel, Checkbox, FormLabel, RadioGroup, Radio, FormGroup, Switch } from '@mui/material';
import Slider from '@mui/material/Slider';
import Divider from '@mui/material/Divider';
import Grid from '@mui/material/Unstable_Grid2';

import PPC from '../components/PPC';

const EPSILON = 1.17549435082228750797e-38;

class App extends Component {
    constructor(props) {
        super(props);
        this.state = {
            axes: {
                "v_1": {
                    label: "Var 1",
                    range: [0, 100],
                    // visibleRange: [20, 70],
                    datums: [...Array(100)].map(() => Math.random() * 100),
                    tickPositions: [0, 25, 50, 75, 100],
                },
                "v_2": {
                    label: "Var 2",
                    range: [15, 30],
                    datums: [...Array(100)].map(() => Math.random() * (30 - 15) + 15)
                },
                "v_3": {
                    label: "Var 3",
                    range: [0, 1],
                    datums: [...Array(100)].map(() => (Math.random()) > 0.5 ? 0.9 : 0.1),
                    tickPositions: [0.1, 0.9],
                    tickLabels: ["False", "True"],
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
                    scale: "plasma",
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
            demo_slider_value_start: EPSILON,
            demo_slider_value_end: 1.0,
            coloring_constant_value: 0.5,
            coloring_attribute: "v_1",
        };
        this.setProps = this.setProps.bind(this);
    }

    setProps(newProps) {
        this.setState(newProps);
    }

    render() {
        const activeLabel = this.state["activeLabel"];
        const demoSliderValueStart = this.state["demo_slider_value_start"];
        const demoSliderValueEnd = this.state["demo_slider_value_end"];
        const colorBarState = this.state["colorBar"];

        let coloringMode;
        switch (typeof (this.state["colors"].selected.color)) {
            case 'number':
                coloringMode = "constant";
                break;
            case 'string':
                coloringMode = "attribute";
            default:
                if (this.state["colors"].selected.color.type === "probability") {
                    coloringMode = "probability";
                }
        }
        const constantColoringValue = this.state["coloring_constant_value"];
        const attributeColoringValue = this.state["coloring_attribute"];
        const colorMapValue = this.state["colors"].selected.scale;

        const debugShowAxisBB = this.state["debug"].showAxisBoundingBox;
        const debugShowLabelBB = this.state["debug"].showLabelBoundingBox;
        const debugShowCurvesBB = this.state["debug"].showCurvesBoundingBox;
        const debugShowAxisLineBB = this.state["debug"].showAxisLineBoundingBox;
        const debugShowSelectionsBB = this.state["debug"].showSelectionsBoundingBox;
        const debugShowColorBarBB = this.state["debug"].showColorBarBoundingBox;

        return (
            <Box style={{ height: "95%", padding: "2rem" }}>
                <Grid container style={{ height: "100%" }} spacing={2}>
                    <Grid xs={10}>
                        <PPC
                            setProps={this.setProps}
                            {...this.state}
                        />
                    </Grid>
                    <Grid xs={2}>
                        <Stack
                            spacing={2}
                            justifyContent="flex-start"
                            alignItems="flex-start"
                            maxHeight={"95vh"}
                            paddingX={"2rem"}
                            style={{ overflow: "auto" }}
                        >
                            <Typography variant='h4'>Labels</Typography>
                            <FormControl fullWidth>
                                <InputLabel>Label</InputLabel>
                                <Select
                                    value={activeLabel}
                                    label="Label"
                                    onChange={e => {
                                        this.setProps({ activeLabel: e.target.value });
                                    }}
                                >
                                    <MenuItem value={"label_1"}>Label 1</MenuItem>
                                    <MenuItem value={"label_2"}>Label 2</MenuItem>
                                    <MenuItem value={"label_3"}>Label 3</MenuItem>
                                </Select>
                            </FormControl>
                            <FormControl fullWidth>
                                <FormLabel>Selection probability bounds</FormLabel>
                                <Slider
                                    min={EPSILON}
                                    max={1.0}
                                    step={EPSILON}
                                    value={[demoSliderValueStart, demoSliderValueEnd]}
                                    onChange={(e, value) => {
                                        const labels = window.structuredClone(this.state["labels"]);
                                        const activeLabel = this.state["activeLabel"];
                                        labels[activeLabel]["selectionBounds"] = value;
                                        this.setProps({ labels, demo_slider_value_start: value[0], demo_slider_value_end: value[1] })
                                    }}
                                />
                            </FormControl>

                            <Divider flexItem />

                            <Typography variant='h4'>Color Scale</Typography>
                            <FormControl fullWidth>
                                <FormControlLabel
                                    control={<Checkbox
                                        checked={colorBarState === "visible"}
                                        onChange={(e, checked) => {
                                            if (checked) {
                                                this.setProps({ colorBar: "visible" })
                                            } else {
                                                this.setProps({ colorBar: "hidden" })
                                            }
                                        }}
                                    />}
                                    label={"Show color scale"}
                                />
                            </FormControl>
                            <FormControl fullWidth>
                                <FormLabel>Coloring</FormLabel>
                                <RadioGroup
                                    value={coloringMode}
                                    onChange={(e, value) => {
                                        const colors = window.structuredClone(this.state["colors"]);
                                        switch (value) {
                                            case "constant":
                                                colors.selected.color = this.state["coloring_constant_value"];
                                                break;
                                            case "attribute":
                                                colors.selected.color = this.state["coloring_attribute"];
                                                break;
                                            case "probability":
                                                colors.selected.color = { type: "probability" };
                                                break;
                                        }
                                        this.setProps({ colors });
                                    }}
                                >
                                    <FormControlLabel
                                        control={<Radio />}
                                        value={"constant"}
                                        label={"Constant color"}
                                    />
                                    <FormControlLabel
                                        control={<Radio />}
                                        value={"attribute"}
                                        label={"By Attribute"}
                                    />
                                    <FormControlLabel
                                        control={<Radio />}
                                        value={"probability"}
                                        label={"By Probability"}
                                    />
                                </RadioGroup>
                            </FormControl>
                            <FormControl fullWidth disabled={coloringMode !== "constant"}>
                                <FormLabel>Coloring constant</FormLabel>
                                <Slider
                                    min={0}
                                    max={1.0}
                                    step={EPSILON}
                                    value={constantColoringValue}
                                    disabled={coloringMode !== "constant"}
                                    onChange={(e, value) => {
                                        const colors = window.structuredClone(this.state["colors"]);
                                        colors.selected.color = value;
                                        this.setProps({ colors, coloring_constant_value: value })
                                    }}
                                />
                            </FormControl>
                            <FormControl fullWidth disabled={coloringMode !== "attribute"}>
                                <FormLabel>Coloring Attribute</FormLabel>
                                <RadioGroup
                                    value={attributeColoringValue}
                                    onChange={(e, value) => {
                                        const colors = window.structuredClone(this.state["colors"]);
                                        colors.selected.color = value;
                                        this.setProps({ colors, coloring_attribute: value });
                                    }}
                                >
                                    <FormControlLabel
                                        control={<Radio />}
                                        value={"v_1"}
                                        label={"Var 1"}
                                    />
                                    <FormControlLabel
                                        control={<Radio />}
                                        value={"v_2"}
                                        label={"Var 2"}
                                    />
                                    <FormControlLabel
                                        control={<Radio />}
                                        value={"v_3"}
                                        label={"Var 3"}
                                    />
                                    <FormControlLabel
                                        control={<Radio />}
                                        value={"v_4"}
                                        label={"Var 4"}
                                    />
                                    <FormControlLabel
                                        control={<Radio />}
                                        value={"v_5"}
                                        label={"Var 5"}
                                    />
                                    <FormControlLabel
                                        control={<Radio />}
                                        value={"v_6"}
                                        label={"Var 6"}
                                    />
                                </RadioGroup>
                            </FormControl>
                            <FormControl fullWidth>
                                <FormLabel>Coloring Map</FormLabel>
                                <RadioGroup
                                    value={colorMapValue}
                                    onChange={(e, value) => {
                                        const colors = window.structuredClone(this.state["colors"]);
                                        colors.selected.scale = value;
                                        this.setProps({ colors })
                                    }}
                                >
                                    <FormControlLabel
                                        control={<Radio />}
                                        value={"magma"}
                                        label={"Magma"}
                                    />
                                    <FormControlLabel
                                        control={<Radio />}
                                        value={"inferno"}
                                        label={"Inferno"}
                                    />
                                    <FormControlLabel
                                        control={<Radio />}
                                        value={"plasma"}
                                        label={"Plasma"}
                                    />
                                    <FormControlLabel
                                        control={<Radio />}
                                        value={"viridis"}
                                        label={"Viridis"}
                                    />
                                </RadioGroup>
                            </FormControl>

                            <Divider flexItem />

                            <Typography variant='h4'>Debug</Typography>
                            <FormGroup
                                onChange={e => {
                                    const element = e.target as HTMLInputElement;
                                    const debug = window.structuredClone(this.state["debug"]);
                                    switch (element.value) {
                                        case "axis":
                                            debug.showAxisBoundingBox = !debug.showAxisBoundingBox;
                                            break;
                                        case "label":
                                            debug.showLabelBoundingBox = !debug.showLabelBoundingBox;
                                            break;
                                        case "curves":
                                            debug.showCurvesBoundingBox = !debug.showCurvesBoundingBox;
                                            break;
                                        case "axis_lines":
                                            debug.showAxisLineBoundingBox = !debug.showAxisLineBoundingBox;
                                            break;
                                        case "selections":
                                            debug.showSelectionsBoundingBox = !debug.showSelectionsBoundingBox;
                                            break;
                                        case "colorbar":
                                            debug.showColorBarBoundingBox = !debug.showColorBarBoundingBox;
                                            break;
                                    }
                                    this.setProps({ debug });
                                }}
                            >
                                <FormLabel>Bounding Boxes</FormLabel>
                                <FormControlLabel control={<Switch checked={debugShowAxisBB} />} value="axis" label="Axis" />
                                <FormControlLabel control={<Switch checked={debugShowLabelBB} />} value="label" label="Label" />
                                <FormControlLabel control={<Switch checked={debugShowCurvesBB} />} value="curves" label="Curves" />
                                <FormControlLabel control={<Switch checked={debugShowAxisLineBB} />} value="axis_lines" label="Axis lines" />
                                <FormControlLabel control={<Switch checked={debugShowSelectionsBB} />} value="selections" label="Selections" />
                                <FormControlLabel control={<Switch checked={debugShowColorBarBB} />} value="colorbar" label="Colorbar" />
                            </FormGroup>
                        </Stack>
                    </Grid>
                </Grid>
            </Box>
        )
    }
}

export default App;
