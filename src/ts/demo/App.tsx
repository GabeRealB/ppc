/* eslint no-magic-numbers: 0 */
import React, { Component } from 'react';
import { v4 as uuid } from 'uuid';

import List from '@mui/material/List';
import Stack from '@mui/material/Stack';
import FormControl from '@mui/material/FormControl';
import InputBase from '@mui/material/InputBase';
import FormControlLabel from '@mui/material/FormControlLabel';
import ListItem from '@mui/material/ListItem';
import ListItemText from '@mui/material/ListItemText';
import ListItemAvatar from '@mui/material/ListItemAvatar';
import IconButton from '@mui/material/IconButton';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Slider from '@mui/material/Slider';
import Divider from '@mui/material/Divider';
import Container from '@mui/material/Container';
import Switch from '@mui/material/Switch';
import FormLabel from '@mui/material/FormLabel';
import RadioGroup from '@mui/material/RadioGroup';
import Radio from '@mui/material/Radio';
import FormGroup from '@mui/material/FormGroup';
import Accordion from '@mui/material/Accordion';
import AccordionDetails from '@mui/material/AccordionDetails';
import AccordionSummary from '@mui/material/AccordionSummary';
import MobileStepper from '@mui/material/MobileStepper';
import Button from '@mui/material/Button';
import KeyboardArrowLeft from '@mui/icons-material/KeyboardArrowLeft';
import KeyboardArrowRight from '@mui/icons-material/KeyboardArrowRight';
import Dialog from '@mui/material/Dialog';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import DialogContentText from '@mui/material/DialogContentText';
import DialogTitle from '@mui/material/DialogTitle';
import Skeleton from '@mui/material/Skeleton';
import Grid from '@mui/material/Unstable_Grid2';

import HelpIcon from '@mui/icons-material/Help';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import VisibilityIcon from '@mui/icons-material/Visibility';
import VisibilityOffIcon from '@mui/icons-material/VisibilityOff';
import PanToolIcon from '@mui/icons-material/PanTool';
import DragHandleIcon from '@mui/icons-material/DragHandle';
import DeleteIcon from '@mui/icons-material/Delete';
import AddIcon from '@mui/icons-material/Add';
import OpenInFullIcon from '@mui/icons-material/OpenInFull';
import OpenWithIcon from '@mui/icons-material/OpenWith';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import StopIcon from '@mui/icons-material/Stop';

import PPC, { Axis, Props } from '../components/PPC';
import { Paper } from '@mui/material';

const EPSILON = 1.17549435082228750797e-38;

type DemoTask = {
    name: string,
    shortDescription: string,
    viewed: boolean,
    canContinue: (Props) => boolean,
};

type DemoState = {
    userId: uuid,
    userGroup: "PC" | "PPC",

    currentTask: number,
    tasks: DemoTask[],
    showInstructions: boolean,

    probabilityRangeStart: number,
    probabilityRangeEnd: number,
    constantColorModeValue: number,
    attributeColorModeValue: string,
    showDebugInfo: boolean,
};

type AppState = {
    ppcState: Props,
    demo: DemoState,
};

class App extends Component<any, AppState> {
    constructor(props) {
        super(props);
        this.setProps = this.setProps.bind(this);

        const searchParams = new URLSearchParams(window.location.search);
        const debugMode = searchParams.has("debug");
        var userGroup = searchParams.get("userGroup");
        if (userGroup !== "PC" && userGroup !== "PPC") {
            userGroup = Math.random() < 0.5 ? "PC" : "PPC";
        }

        this.state = {
            ppcState: {
                axes: {
                    "v_1": {
                        label: "Var 1",
                        range: [0, 100],
                        // visibleRange: [20, 70],
                        dataPoints: [...Array(100)].map(() => Math.random() * 100),
                        tickPositions: [0, 25, 50, 75, 100],
                    },
                    "v_2": {
                        label: "Var 2",
                        range: [15, 30],
                        dataPoints: [...Array(100)].map(() => Math.random() * (30 - 15) + 15)
                    },
                    "v_3": {
                        label: "Var 3",
                        range: [0, 1],
                        dataPoints: [...Array(100)].map(() => (Math.random()) > 0.5 ? 0.9 : 0.1),
                        tickPositions: [0.1, 0.9],
                        tickLabels: ["False", "True"],
                    },
                    "v_4": {
                        label: "Var 4",
                        range: [15, 30],
                        dataPoints: [...Array(100)].map(() => Math.random() * (30 - 15) + 15)
                    },
                    "v_5": {
                        label: "Var 5",
                        range: [15, 30],
                        dataPoints: [...Array(100)].map(() => Math.random() * (30 - 15) + 15)
                    },
                    "v_6": {
                        label: "Var 6",
                        range: [15, 30],
                        dataPoints: [...Array(100)].map(() => Math.random() * (30 - 15) + 15),
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
                setProps: this.setProps,
            },
            demo: {
                userId: uuid(),
                userGroup: userGroup as "PC" | "PPC",
                showInstructions: true,
                currentTask: 0,
                tasks: [
                    { name: "Task 1 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                    { name: "Task 2 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                    { name: "Task 3 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => false },
                    { name: "Task 4 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                    { name: "Task 5 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                    { name: "Task 6 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                    { name: "Task 7 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                    { name: "Task 8 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                    { name: "Task 9 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                    { name: "Task 10 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                    { name: "Task 11 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                    { name: "Task 12 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                    { name: "Task 13 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                    { name: "Task 14 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                    { name: "Task 15 name", shortDescription: "Lorem ipsum dolor sit amet.", viewed: false, canContinue: () => true },
                ],
                probabilityRangeStart: EPSILON,
                probabilityRangeEnd: 1.0,
                constantColorModeValue: 0.5,
                attributeColorModeValue: "v_1",
                showDebugInfo: debugMode
            },
        };
    }

    setProps(newProps) {
        this.setState(newProps);
    }

    render() {
        const {
            ppcState,
            demo,
        } = this.state;
        const {
            axes,
        } = ppcState;

        return (
            <Box style={{ height: "95%", padding: "2rem" }}>
                <Grid container style={{ height: "100%" }} spacing={2}>
                    <Grid xs={10}>
                        <PPC
                            {...ppcState}
                        />
                    </Grid>
                    <Grid xs={2} maxHeight={"95%"} sx={{ overflow: "auto" }}>
                        <Stack
                            spacing={2}
                            justifyContent="flex-start"
                            alignItems="flex-start"
                            paddingX={"2rem"}
                        >
                            {InstructionsDialog(demo, this.setProps)}

                            {TaskView(ppcState, demo, this.setProps)}

                            <Divider flexItem />

                            <div>
                                {AttributeList(ppcState, axes, this.setProps)}
                                {ColorSettings(ppcState, demo, this.setProps)}
                                {ActionsInfo(demo)}
                                {DebugInfo(ppcState, demo, this.setProps)}
                            </div>

                            <Divider flexItem />

                            {LabelsView(ppcState, demo, this.setProps)}
                        </Stack>
                    </Grid>
                </Grid>
            </Box>
        )
    }
}

export default App;

const InstructionsDialog = (demo: DemoState, setProps: (newProps) => void) => {
    const { showInstructions, currentTask, tasks } = demo;
    const task = tasks[currentTask];
    const { name } = task;

    const handleClose = () => {
        demo.showInstructions = false;
        setProps({ demo });
    };

    return (
        <Dialog
            onClose={handleClose}
            open={showInstructions}
            aria-labelledby="instructions-dialog-title"
            aria-describedby="instructions-dialog-description"
        >
            <DialogTitle id="instructions-dialog-title">
                {name}
            </DialogTitle>
            <DialogContent id="instructions-dialog-description">
                <Skeleton variant='rectangular' animation={false} height={480} />
                <DialogContentText>
                    Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
                </DialogContentText>
            </DialogContent>
            <DialogActions>
                <Button onClick={handleClose} autoFocus>
                    Close
                </Button>
            </DialogActions>
        </Dialog>
    )
}

const TaskView = (ppc: Props, demo: DemoState, setProps: (newProps) => void) => {
    const { currentTask, tasks } = demo;
    const task = tasks[currentTask];
    const { name, shortDescription } = task;

    const handleNext = () => {
        demo.currentTask = currentTask + 1;
        demo.showInstructions = !tasks[currentTask + 1].viewed;
        setProps({ demo });
    };

    const handleBack = () => {
        demo.currentTask = currentTask - 1;
        setProps({ demo });
    };

    const openInstructionsDialog = () => {
        demo.showInstructions = true;
        setProps({ demo });
    }

    if (!task.viewed) {
        task.viewed = true;
        setProps({ demo });
    }

    return (
        <Box width={"100%"}>
            <Typography variant='h5'>{name}</Typography>
            <Typography variant='subtitle1'>{shortDescription}</Typography>
            <Container>
                <Button
                    variant="contained"
                    startIcon={<HelpIcon />}
                    sx={{ width: "100%" }}
                    onClick={openInstructionsDialog}
                >
                    Show Instructions
                </Button>
            </Container>
            <MobileStepper
                variant="progress"
                steps={tasks.length}
                position="static"
                activeStep={currentTask}
                nextButton={
                    <Button size="small" onClick={handleNext} disabled={currentTask === tasks.length - 1 || !task.canContinue(ppc)}>
                        Next
                        <KeyboardArrowRight />
                    </Button>
                }
                backButton={
                    <Button size="small" onClick={handleBack} disabled={currentTask === 0}>
                        <KeyboardArrowLeft />
                        Previous
                    </Button>
                }
            />
        </Box>
    )
}

const AttributeListItem = (key: string, axis: Axis, update: (Axis) => void) => {
    const visible = axis.hidden !== true;
    const icon = visible ? <VisibilityIcon /> : <VisibilityOffIcon />;

    const handle_click = () => {
        const new_axis = window.structuredClone(axis);
        new_axis.hidden = visible;
        update(new_axis);
    }

    return (
        <ListItem key={key}>
            <IconButton
                edge="start"
                onClick={handle_click}
            >
                {icon}
            </IconButton>
            <ListItemText primary={axis.label} />
        </ListItem>
    );
}

const AttributeList = (ppcState: Props, axes: { [id: string]: Axis }, setProps: (newProps) => void) => {
    const items = [];
    for (const key in axes) {
        const update = (axis: Axis) => {
            const new_axes = window.structuredClone(axes);
            new_axes[key] = axis;
            ppcState.axes = new_axes;
            setProps({ ppcState });
        };
        const axis = axes[key];
        items.push(AttributeListItem(key, axis, update));
    }

    return (
        <Accordion sx={{ width: "100%" }}>
            <AccordionSummary
                id="attributes-header"
                aria-controls='attributes-content'
                expandIcon={<ExpandMoreIcon />}
            >
                <Typography variant='h5'>Attributes</Typography>
            </AccordionSummary>
            <AccordionDetails>
                <List dense>
                    {items}
                </List>
            </AccordionDetails>
        </Accordion>
    );
}

const ColorSettings = (ppc: Props, demo: DemoState, setProps: (newProps) => void) => {
    const { userGroup, constantColorModeValue, attributeColorModeValue } = demo;
    const { colors, colorBar, axes } = ppc;

    let colorMode;
    switch (typeof (colors.selected.color)) {
        case 'number':
            colorMode = "constant";
            break;
        case 'string':
            colorMode = "attribute";
        default:
            if (typeof (colors.selected.color) == 'object'
                && colors.selected.color.type === "probability") {
                colorMode = "probability";
            }
    }
    const colorMapValue = colors.selected.scale as string;

    const switchColorBarVisibility = (e, visibility) => {
        ppc.colorBar = visibility;
        setProps({ ppcState: ppc })
    };

    const switchColorMode = (e, colorMode) => {
        const colors_clone = window.structuredClone(colors);
        switch (colorMode) {
            case "constant":
                colors_clone.selected.color = constantColorModeValue;
                break;
            case "attribute":
                colors_clone.selected.color = attributeColorModeValue;
                break;
            case "probability":
                colors_clone.selected.color = { type: "probability" };
                break;
        }
        ppc.colors = colors_clone;
        setProps({ ppcState: ppc });
    };

    const setConstantColorValue = (e, value) => {
        const colors_clone = window.structuredClone(colors);
        colors_clone.selected.color = value as number;
        ppc.colors = colors_clone;
        demo.constantColorModeValue = value as number;
        setProps({ ppcState: ppc, demo })
    };

    const setAttributeColorValue = (e, value) => {
        const colors_clone = window.structuredClone(colors);
        colors_clone.selected.color = value;
        ppc.colors = colors_clone;
        demo.attributeColorModeValue = value;
        setProps({ ppcState: ppc, demo })
    };

    const setColorMap = (e, colorMap) => {
        const colors_clone = window.structuredClone(colors);
        colors_clone.selected.scale = colorMap;
        ppc.colors = colors_clone;
        setProps({ ppcState: ppc })
    };

    return (
        <Accordion sx={{ width: "100%" }}>
            <AccordionSummary
                id="color-settings-header"
                aria-controls='color-settings-content'
                expandIcon={<ExpandMoreIcon />}
            >
                <Typography variant='h5'>Colors</Typography>
            </AccordionSummary>
            <AccordionDetails>
                <FormControl fullWidth>
                    <FormLabel id="color-settings-color-bar-group-label">Color Bar</FormLabel>
                    <RadioGroup
                        row
                        aria-labelledby="color-settings-color-bar-group-label"
                        name="color-settings-color-bar-group"
                        value={colorBar}
                        onChange={switchColorBarVisibility}
                    >
                        <FormControlLabel value="hidden" control={<Radio />} label="Hidden" />
                        <FormControlLabel value="visible" control={<Radio />} label="Visible" />
                    </RadioGroup>
                </FormControl>
                <FormControl fullWidth>
                    <FormLabel>Color Mode</FormLabel>
                    <RadioGroup
                        row
                        value={colorMode}
                        onChange={switchColorMode}
                    >
                        <FormControlLabel
                            control={<Radio />}
                            value={"constant"}
                            label={"Const."}
                        />
                        <FormControlLabel
                            control={<Radio />}
                            value={"attribute"}
                            label={"Attr."}
                        />
                        {userGroup === "PPC" ? <FormControlLabel
                            control={<Radio />}
                            value={"probability"}
                            label={"Prob."}
                        /> : null}
                    </RadioGroup>
                </FormControl>
                {
                    colorMode === "constant" ?
                        <FormControl fullWidth disabled={colorMode !== "constant"}>
                            <FormLabel>Color Mode: Constant</FormLabel>
                            <Slider
                                min={0}
                                max={1.0}
                                step={EPSILON}
                                value={constantColorModeValue}
                                onChange={setConstantColorValue}
                            />
                        </FormControl> : null
                }
                {
                    colorMode === "attribute" ?
                        <FormControl fullWidth>
                            <FormLabel>Color Mode: Attribute</FormLabel>
                            <RadioGroup
                                row
                                value={attributeColorModeValue}
                                onChange={setAttributeColorValue}
                            >
                                {Object.entries(axes).map(([k, v]) => <FormControlLabel
                                    control={<Radio />}
                                    value={k}
                                    label={v.label}
                                />)}
                            </RadioGroup>
                        </FormControl> : null
                }
                <FormControl fullWidth>
                    <FormLabel>Color Map</FormLabel>
                    <RadioGroup
                        row
                        value={colorMapValue}
                        onChange={setColorMap}
                    >
                        {["Magma", "Inferno", "Plasma", "Viridis"].map((v) => <FormControlLabel
                            control={<Radio />}
                            value={v.toLowerCase()}
                            label={v}
                        />)}
                    </RadioGroup>
                </FormControl>
            </AccordionDetails>
        </Accordion>
    );
}

const ActionsInfo = (demo: DemoState) => {
    const { userGroup } = demo;

    const sharedActions = [
        { icon: <PanToolIcon />, desc: "Move attribute axis", sec: "Left mouse button on axis label" },
        { icon: <AddIcon />, desc: "Create selection", sec: "Left mouse button on empty axis line" },
        { icon: <DragHandleIcon />, desc: "Move selection", sec: "Left mouse button on selection" },
        { icon: <DeleteIcon />, desc: "Delete selection", sec: "Left click on selection" },
    ];
    const ppcActions = [
        { icon: <OpenInFullIcon />, desc: "Expand/Collapse axis", sec: "Left click on axis label" },
        { icon: <OpenWithIcon />, desc: "Move control point", sec: "Left mouse button on selection control point" },
        { icon: <DeleteIcon />, desc: "Delete control point", sec: "Left click on selection control point" },
    ];

    return (
        <Accordion sx={{ width: "100%" }}>
            <AccordionSummary
                id="actions-info-header"
                aria-controls='actions-info-content'
                expandIcon={<ExpandMoreIcon />}
            >
                <Typography variant='h5'>Actions</Typography>
            </AccordionSummary>
            <AccordionDetails>
                <List dense>
                    {sharedActions.map((a) => <ListItem>
                        <ListItemAvatar>
                            {a.icon}
                        </ListItemAvatar>
                        <ListItemText
                            primary={a.desc}
                            secondary={a.sec}
                        />
                    </ListItem>)}
                    {userGroup === 'PPC' ? ppcActions.map((a) => <ListItem>
                        <ListItemAvatar>
                            {a.icon}
                        </ListItemAvatar>
                        <ListItemText
                            primary={a.desc}
                            secondary={a.sec}
                        />
                    </ListItem>) : null}
                </List>
            </AccordionDetails>
        </Accordion>
    );
}

const DebugInfo = (ppc: Props, demo: DemoState, setProps: (newProps) => void) => {
    const { userId, userGroup, currentTask, tasks } = demo;

    const debugShowAxisBB = ppc.debug.showAxisBoundingBox;
    const debugShowLabelBB = ppc.debug.showLabelBoundingBox;
    const debugShowCurvesBB = ppc.debug.showCurvesBoundingBox;
    const debugShowAxisLineBB = ppc.debug.showAxisLineBoundingBox;
    const debugShowSelectionsBB = ppc.debug.showSelectionsBoundingBox;
    const debugShowColorBarBB = ppc.debug.showColorBarBoundingBox;

    var debug_item = null;
    if (demo.showDebugInfo) {
        debug_item = (
            <Accordion sx={{ width: "100%" }}>
                <AccordionSummary
                    id="debug-info-header"
                    aria-controls='debug-info-content'
                    expandIcon={<ExpandMoreIcon />}
                >
                    <Typography variant='h5'>Debug Info</Typography>
                </AccordionSummary>
                <AccordionDetails>
                    <Typography variant='body1'>Id: {userId}</Typography>
                    <Typography variant='body1'>Group: {userGroup}</Typography>
                    <Typography variant='body1'>Task: {currentTask + 1}</Typography>
                    <Typography variant='body1'>Num Tasks: {tasks.length}</Typography>
                    <Divider flexItem />
                    <FormGroup
                        onChange={e => {
                            const element = e.target as HTMLInputElement;
                            const debug_clone = window.structuredClone(ppc.debug);
                            switch (element.value) {
                                case "axis":
                                    debug_clone.showAxisBoundingBox = !debug_clone.showAxisBoundingBox;
                                    break;
                                case "label":
                                    debug_clone.showLabelBoundingBox = !debug_clone.showLabelBoundingBox;
                                    break;
                                case "curves":
                                    debug_clone.showCurvesBoundingBox = !debug_clone.showCurvesBoundingBox;
                                    break;
                                case "axis_lines":
                                    debug_clone.showAxisLineBoundingBox = !debug_clone.showAxisLineBoundingBox;
                                    break;
                                case "selections":
                                    debug_clone.showSelectionsBoundingBox = !debug_clone.showSelectionsBoundingBox;
                                    break;
                                case "colorbar":
                                    debug_clone.showColorBarBoundingBox = !debug_clone.showColorBarBoundingBox;
                                    break;
                            }
                            ppc.debug = debug_clone;
                            setProps({ ppcState: ppc });
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
                </AccordionDetails>
            </Accordion>
        )
    }

    return debug_item;
}

const LabelsViewItem = (active: boolean, name: string, deleteLabel: () => void, toggleActive: () => void) => {
    return (
        <Paper
            sx={{ display: 'flex', alignItems: 'center' }}
        >
            <IconButton onClick={toggleActive}>
                {active ? <StopIcon /> : <PlayArrowIcon />}
            </IconButton>
            <Typography
                sx={{ ml: 1, flex: 1 }}
                variant='body1'
            >
                {name}
            </Typography>
            <Divider sx={{ height: 28, m: 0.5 }} orientation="vertical" />
            <IconButton onClick={deleteLabel}>
                <DeleteIcon />
            </IconButton>
        </Paper>
    )
}

const LabelsView = (ppc: Props, demo: DemoState, setProps: (newProps) => void) => {
    const { labels, activeLabel } = ppc;
    const { probabilityRangeStart, probabilityRangeEnd } = demo;

    const items = Object.entries(labels).map(([k, v]) => {
        const deleteLabel = () => {
            const labels_clone = window.structuredClone(labels);
            delete labels_clone[k];
            ppc.labels = labels_clone;
            if (activeLabel === k) {
                const keys = Object.keys(labels_clone);
                if (keys.length == 0) {
                    ppc.activeLabel = null;
                } else {
                    ppc.activeLabel = keys[keys.length - 1];
                    const new_active_label = labels_clone[ppc.activeLabel];
                    if (new_active_label.selectionBounds) {
                        demo.probabilityRangeStart = new_active_label.selectionBounds[0];
                        demo.probabilityRangeEnd = new_active_label.selectionBounds[1];
                    } else {
                        demo.probabilityRangeStart = 0.0;
                        demo.probabilityRangeEnd = 1.0;
                    }
                }
            }
            setProps({ ppcState: ppc, demo });
        }
        const toggleActive = () => {
            ppc.activeLabel = k;
            if (v.selectionBounds) {
                demo.probabilityRangeStart = v.selectionBounds[0];
                demo.probabilityRangeEnd = v.selectionBounds[1];
            } else {
                demo.probabilityRangeStart = 0.0;
                demo.probabilityRangeEnd = 1.0;
            }
            setProps({ ppcState: ppc, demo });
        }

        return LabelsViewItem(k === activeLabel, k, deleteLabel, toggleActive);
    });

    const handleProbabilityRangeChange = (e, range) => {
        const labels_clone = window.structuredClone(labels);
        labels_clone[activeLabel].selectionBounds = range as [number, number];
        ppc.labels = labels_clone;
        demo.probabilityRangeStart = range[0];
        demo.probabilityRangeEnd = range[1];
        setProps({ ppcState: ppc, demo })
    };

    return (
        <Box width={"100%"}>
            <Typography variant='h5'>Labels</Typography>
            <Stack spacing={1}>
                {items}

                <Paper
                    component="form"
                    sx={{ display: 'flex', alignItems: 'center' }}
                >
                    <InputBase
                        sx={{ ml: 1, flex: 1 }}
                        placeholder="New Label Name"
                    />
                    <IconButton type="button" sx={{ p: '10px' }}>
                        <AddIcon />
                    </IconButton>
                </Paper>

                {activeLabel ?
                    <FormControl>
                        <FormLabel>Selection probability bounds</FormLabel>
                        <Slider
                            min={EPSILON}
                            max={1.0}
                            step={EPSILON}
                            value={[probabilityRangeStart, probabilityRangeEnd]}
                            onChange={handleProbabilityRangeChange}
                            valueLabelDisplay="auto"
                            size="small"
                        />
                    </FormControl> : null}
            </Stack>
        </Box>
    )
}
