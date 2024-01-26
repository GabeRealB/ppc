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
import Paper from '@mui/material/Paper';
import Grid from '@mui/material/Unstable_Grid2';

import HelpIcon from '@mui/icons-material/Help';
import RestartAltIcon from '@mui/icons-material/RestartAlt';
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
import InfoIcon from '@mui/icons-material/Info';

import moveAxesInstr from './resources/move_axes_instr.mp4'

import PPC from '../components/PPC';
import { Axis, Props, InteractionMode} from '../types'

const EPSILON = 1.17549435082228750797e-38;

type DemoTask = {
    name: string,
    shortDescription: string,
    instructions: () => React.JSX.Element,
    viewed: boolean,
    initialState: Props,
    finalState: Props,
    canContinue: (ppc: Props) => boolean,
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
        this.setPPCProps = this.setPPCProps.bind(this);

        const searchParams = new URLSearchParams(window.location.search);
        const debugMode = searchParams.has("debug");
        let userGroup = searchParams.get("userGroup");
        if (userGroup !== "PC" && userGroup !== "PPC") {
            userGroup = Math.random() < 0.5 ? "PC" : "PPC";
        }

        const tasks = constructTasks(userGroup as "PC" | "PPC");
        const ppc = window.structuredClone(tasks[0].initialState);
        ppc.setProps = this.setPPCProps;

        this.state = {
            ppcState: ppc,
            demo: {
                userId: uuid(),
                userGroup: userGroup as "PC" | "PPC",
                showInstructions: true,
                currentTask: 0,
                tasks: tasks,
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

    setPPCProps(newProps) {
        const { ppcState } = this.state;
        for (const [k, v] of Object.entries(newProps)) {
            if (k in ppcState) {
                ppcState[k] = v;
            }
        }

        this.setProps({ ppcState });
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
                            {LabelsView(ppcState, demo, this.setProps)}
                            <Divider flexItem />

                            <div>
                                {AttributeList(ppcState, axes, this.setProps)}
                                {ColorSettings(ppcState, demo, this.setProps)}
                                {ActionsInfo(ppcState)}
                                {DebugInfo(ppcState, demo, this.setProps)}
                            </div>
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
                {task.instructions()}
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
        const current = tasks[currentTask];
        const next = tasks[currentTask + 1];

        const setPpcProps = ppc.setProps;
        ppc.setProps = undefined;
        current.finalState = window.structuredClone(ppc);

        let nextPpc = window.structuredClone(next.finalState);
        if (!nextPpc) {
            nextPpc = window.structuredClone(next.initialState);
        }
        nextPpc.setProps = setPpcProps;

        demo.currentTask = currentTask + 1;
        demo.showInstructions = !next.viewed;
        setProps({ ppcState: nextPpc, demo });
    };

    const handleBack = () => {
        const current = tasks[currentTask];
        const prev = tasks[currentTask - 1];

        const setPpcProps = ppc.setProps;
        ppc.setProps = undefined;
        current.finalState = window.structuredClone(ppc);

        const prevPpc = window.structuredClone(prev.finalState);
        prevPpc.setProps = setPpcProps;

        demo.currentTask = currentTask - 1;
        setProps({ ppcState: prevPpc, demo });
    };

    const handleReset = () => {
        const current = tasks[currentTask];

        const newPPC = window.structuredClone(current.initialState);
        newPPC.setProps = ppc.setProps;
        setProps({ ppcState: newPPC });
    }

    const openInstructionsDialog = () => {
        demo.showInstructions = true;
        setProps({ demo });
    }

    if (!task.viewed) {
        task.viewed = true;
        setProps({ demo });
    }

    return (
        <Stack width={"100%"} spacing={1}>
            <Typography variant='h5'>{name}</Typography>
            <Typography variant='subtitle1'>{shortDescription}</Typography>
            <Container>
                <Button
                    variant="contained"
                    startIcon={<HelpIcon />}
                    sx={{ width: "95%" }}
                    onClick={openInstructionsDialog}
                >
                    Instructions
                </Button>
            </Container>
            <Container>
                <Button
                    variant="contained"
                    startIcon={<RestartAltIcon />}
                    sx={{ width: "95%" }}
                    onClick={handleReset}
                >
                    Reset
                </Button>
            </Container>
            <MobileStepper
                variant="progress"
                steps={tasks.length}
                position="bottom"
                activeStep={currentTask}
                nextButton={
                    <Button size="small" onClick={handleNext} disabled={currentTask === tasks.length - 1 || !task.canContinue(ppc)}>
                        {currentTask !== tasks.length - 1 ? "Next" : "Finish"}
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
        </Stack>
    )
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
            <IconButton onClick={deleteLabel} disabled={deleteLabel == null}>
                <DeleteIcon />
            </IconButton>
        </Paper>
    )
}

const LabelsView = (ppc: Props, demo: DemoState, setProps: (newProps) => void) => {
    const { labels, activeLabel } = ppc;
    const { probabilityRangeStart, probabilityRangeEnd } = demo;
    const interactionMode = ppc.interactionMode ? ppc.interactionMode : InteractionMode.Full;

    const items = Object.entries(labels).map(([k, v]) => {
        const deleteLabel = interactionMode == InteractionMode.Compatibility
            || interactionMode == InteractionMode.Full
            ? () => {
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
            } : null;
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
                <div />
                {items}

                {interactionMode == InteractionMode.Compatibility
                    || interactionMode == InteractionMode.Full
                    ? <Paper
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
                    </Paper> : undefined}

                {activeLabel && (interactionMode == InteractionMode.Full) ?
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
                    </FormControl> : undefined}
            </Stack>
        </Box>
    )
}

const AttributeListItem = (key: string, axis: Axis, update: (Axis) => void) => {
    const { label, hidden } = axis;
    const visible = hidden !== true;

    const handle_click = () => {
        const new_axis = window.structuredClone(axis);
        new_axis.hidden = visible;
        update(new_axis);
    }

    return (
        <Paper
            sx={{ display: 'flex', alignItems: 'center' }}
        >
            <Typography
                sx={{ ml: 1, flex: 1 }}
                variant='body1'
            >
                {label}
            </Typography>
            <Divider sx={{ height: 28, m: 0.5 }} orientation="vertical" />
            <IconButton onClick={handle_click} disabled={update == null}>
                {visible ? <VisibilityIcon /> : <VisibilityOffIcon />}
            </IconButton>
        </Paper>
    );
}

const AttributeList = (ppcState: Props, axes: { [id: string]: Axis }, setProps: (newProps) => void) => {
    const interactionMode = ppcState.interactionMode ? ppcState.interactionMode : InteractionMode.Full;

    const items = [];
    for (const key in axes) {
        const update = interactionMode == InteractionMode.Compatibility
            || interactionMode == InteractionMode.Full
            ? (axis: Axis) => {
                const new_axes = window.structuredClone(axes);
                new_axes[key] = axis;
                ppcState.axes = new_axes;

                if (ppcState.order) {
                    const order_clone = window.structuredClone(ppcState.order);
                    if (axis.hidden) {
                        const index = order_clone.indexOf(key);
                        if (index > -1) {
                            order_clone.splice(index, 1);
                        }
                    } else {
                        order_clone.push(key);
                    }
                    ppcState.order = order_clone;
                }

                setProps({ ppcState });
            } : null;
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
                <Stack width={"100%"} spacing={1}>
                    <Container>
                        <Button
                            variant="contained"
                            startIcon={<InfoIcon />}
                            sx={{ width: "95%" }}
                        >
                            Dataset
                        </Button>
                    </Container>
                    <div />

                    {items}
                </Stack>
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

const ActionsInfo = (ppc: Props) => {
    const { interactionMode } = ppc;

    const actions = [
        {
            icon: <PanToolIcon />, desc: "Move attribute axis", sec: "Left mouse button on axis label",
            modes_check: (m: InteractionMode) => m != InteractionMode.Disabled
        },
        {
            icon: <AddIcon />, desc: "Create selection", sec: "Left mouse button on empty axis line",
            modes_check: (m: InteractionMode) => m == InteractionMode.Compatibility || m == InteractionMode.Full
        },
        {
            icon: <DragHandleIcon />, desc: "Move selection", sec: "Left mouse button on selection",
            modes_check: (m: InteractionMode) => m == InteractionMode.Compatibility || m == InteractionMode.Full
        },
        {
            icon: <DeleteIcon />, desc: "Delete selection", sec: "Left click on selection",
            modes_check: (m: InteractionMode) => m == InteractionMode.Compatibility || m == InteractionMode.Full
        },
        {
            icon: <OpenInFullIcon />, desc: "Expand/Collapse axis", sec: "Left click on axis label",
            modes_check: (m: InteractionMode) => m == InteractionMode.Restricted || m == InteractionMode.Full
        },
        {
            icon: <OpenWithIcon />, desc: "Move control point", sec: "Left mouse button on selection control point",
            modes_check: (m: InteractionMode) => m == InteractionMode.Full
        },
        {
            icon: <DeleteIcon />, desc: "Delete control point", sec: "Left click on selection control point",
            modes_check: (m: InteractionMode) => m == InteractionMode.Full
        },
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
                    {actions.map((a) => a.modes_check(interactionMode) ? <ListItem>
                        <ListItemAvatar>
                            {a.icon}
                        </ListItemAvatar>
                        <ListItemText
                            primary={a.desc}
                            secondary={a.sec}
                        />
                    </ListItem> : null)}
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

    let debug_item = null;
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

const constructTasks = (userGroup: "PC" | "PPC") => {
    return [
        task_0(userGroup),
        task_1(userGroup),
    ];
}

const task_0 = (userGroup: "PC" | "PPC"): DemoTask => {
    const interactionMode = userGroup === "PC"
        ? InteractionMode.RestrictedCompatibility
        : InteractionMode.Restricted;

    const buildInstructions = () => {
        return (
            <Stack spacing={1}>
                <video autoPlay loop muted>
                    <source src={moveAxesInstr} type='video/mp4'></source>
                </video>
                <DialogContentText>
                    Attribute axes can be moved by holding the left mouse button on the label of an attribute.
                    Move the axis of the attribute <b>A1</b>, such that it lies in between the attribute axes
                    &#32;<b>A2</b> and <b>A3</b>.
                    <br /><br />
                    Press the <b>Next</b> button on the bottom right once the task has been completed.
                </DialogContentText>
            </Stack>);
    }

    return {
        name: "Reorder attribute axes.",
        shortDescription: "Reorder A1 in between A2 and A3",
        instructions: buildInstructions,
        viewed: true,
        initialState: {
            axes: {
                "a1": {
                    label: "A1",
                    range: [0, 10],
                    dataPoints: [...Array(100)].map(() => Math.random() * 10),
                },
                "a2": {
                    label: "A2",
                    range: [0, 10],
                    dataPoints: [...Array(100)].map(() => Math.random() * 10),
                },
                "a3": {
                    label: "A3",
                    range: [0, 10],
                    dataPoints: [...Array(100)].map(() => Math.random() * 10),
                },
            },
            order: ["a1", "a2", "a3"],
            labels: {
                "Default": {},
            },
            activeLabel: "Default",
            colors: {
                selected: {
                    scale: "plasma",
                    color: 0.5,
                }
            },
            colorBar: "hidden",
            interactionMode: interactionMode,
            debug: {
                showAxisBoundingBox: false,
                showLabelBoundingBox: false,
                showCurvesBoundingBox: false,
                showAxisLineBoundingBox: false,
                showSelectionsBoundingBox: false,
                showColorBarBoundingBox: false,
            },
            setProps: undefined,
        },
        finalState: null,
        canContinue: (ppc: Props) => (ppc.order[0] == "a2"
            && ppc.order[1] == "a1" && ppc.order[2] == "a3")
            || (ppc.order[0] == "a3" && ppc.order[1] == "a1"
                && ppc.order[2] == "a2")
    };
}

const task_1 = (userGroup: "PC" | "PPC"): DemoTask => {
    const interactionMode = userGroup === "PC"
        ? InteractionMode.Compatibility
        : InteractionMode.Full;

    const buildInstructions = () => {
        return (
            <>
                <Skeleton variant='rectangular' animation={false} height={480} />
                <DialogContentText>
                    Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
                </DialogContentText>
            </>);
    }

    return {
        name: "Task 2",
        shortDescription: "Todo",
        instructions: buildInstructions,
        viewed: false,
        initialState: {
            axes: {
                "a1": {
                    label: "A1",
                    range: [0, 10],
                    dataPoints: [...Array(100)].map(() => Math.random() * 10),
                },
                "a2": {
                    label: "A2",
                    range: [0, 10],
                    dataPoints: [...Array(100)].map(() => Math.random() * 10),
                },
                "a3": {
                    label: "A3",
                    range: [0, 10],
                    dataPoints: [...Array(100)].map(() => Math.random() * 10),
                },
            },
            order: ["a3", "a2", "a1"],
            labels: {
                "Default": {},
                "Default2": {},
            },
            activeLabel: "Default",
            colors: {
                selected: {
                    scale: "plasma",
                    color: 0.5,
                }
            },
            colorBar: "hidden",
            interactionMode: interactionMode,
            debug: {
                showAxisBoundingBox: false,
                showLabelBoundingBox: false,
                showCurvesBoundingBox: false,
                showAxisLineBoundingBox: false,
                showSelectionsBoundingBox: false,
                showColorBarBoundingBox: false,
            },
            setProps: undefined,
        },
        finalState: null,
        canContinue: (ppc: Props) => ppc.order[0] == "a2"
            && ppc.order[1] == "a1"
            && ppc.order[2] == "a3"
    };
}