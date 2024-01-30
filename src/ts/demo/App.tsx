/* eslint no-magic-numbers: 0 */
import React, { Component, createElement, useState } from 'react';
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
import Alert from '@mui/material/Alert';
import Checkbox from '@mui/material/Checkbox';
import Select from '@mui/material/Select';
import InputLabel from '@mui/material/InputLabel';
import MenuItem from '@mui/material/MenuItem';
import Rating from '@mui/material/Rating';

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
import StarIcon from '@mui/icons-material/Star';

import moveAxesInstr from './resources/move_axes_instr.mp4'

import PPC from '../components/PPC';
import { Axis, Props, InteractionMode } from '../types'

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

type UserGroup = "PC" | "PPC";

type DemoPage = "welcome"
    | "demo1"
    | "demo2"
    | "demo3"
    | "finish";

type LevelOfEducation = "Childhood"
    | "Primary"
    | "LowerSecondary"
    | "UpperSecondary"
    | "Post-secondary"
    | "Tertiary"
    | "Bachelor"
    | "Master"
    | "Doctoral"

type Proficiency = "NA"
    | "Fundamental"
    | "Novice"
    | "Intermediate"
    | "Advanced"
    | "Expert"

type Results = {
    education?: LevelOfEducation,
    analysisProficiency?: Proficiency,
    pcProficiency?: Proficiency,
};

type DemoState = {
    currentPage: DemoPage,

    userId: uuid,
    userGroup: UserGroup,

    currentTask: number,
    tasks: DemoTask[],
    showInstructions: boolean,

    showDebugInfo: boolean,

    results: Results,
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

        const tasks = constructTasks(userGroup as UserGroup);
        const ppc = window.structuredClone(tasks[0].initialState);
        ppc.setProps = this.setPPCProps;

        this.state = {
            ppcState: ppc,
            demo: {
                currentPage: "welcome",
                userId: uuid(),
                userGroup: userGroup as UserGroup,
                showInstructions: true,
                currentTask: 0,
                tasks: tasks,
                showDebugInfo: debugMode,
                results: {}
            },
        };
    }

    setProps(newProps) {
        this.setState(newProps);
    }

    setPPCProps(newProps) {
        const { ppcState } = this.state;
        for (const [k, v] of Object.entries(newProps)) {
            ppcState[k] = v;
        }

        this.setProps({ ppcState });
    }

    render() {
        let page = null;
        switch (this.state.demo.currentPage) {
            case 'welcome':
                page = createElement(WelcomePage, this);
                break;
            case 'demo1':
                page = createElement(DemoPage1, this);
                break;
            case 'demo3':
                page = DemoPage3(this);
                break;
            case 'finish':
                break;
        }

        return page;
    }
}

export default App;

function WelcomePage(app: App) {
    const hasWebgpu = !!navigator.gpu;
    const isMobile = (() => {
        let check = false;
        (function (a) { if (/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino|android|ipad|playbook|silk/i.test(a) || /1207|6310|6590|3gso|4thp|50[1-6]i|770s|802s|a wa|abac|ac(er|oo|s\-)|ai(ko|rn)|al(av|ca|co)|amoi|an(ex|ny|yw)|aptu|ar(ch|go)|as(te|us)|attw|au(di|\-m|r |s )|avan|be(ck|ll|nq)|bi(lb|rd)|bl(ac|az)|br(e|v)w|bumb|bw\-(n|u)|c55\/|capi|ccwa|cdm\-|cell|chtm|cldc|cmd\-|co(mp|nd)|craw|da(it|ll|ng)|dbte|dc\-s|devi|dica|dmob|do(c|p)o|ds(12|\-d)|el(49|ai)|em(l2|ul)|er(ic|k0)|esl8|ez([4-7]0|os|wa|ze)|fetc|fly(\-|_)|g1 u|g560|gene|gf\-5|g\-mo|go(\.w|od)|gr(ad|un)|haie|hcit|hd\-(m|p|t)|hei\-|hi(pt|ta)|hp( i|ip)|hs\-c|ht(c(\-| |_|a|g|p|s|t)|tp)|hu(aw|tc)|i\-(20|go|ma)|i230|iac( |\-|\/)|ibro|idea|ig01|ikom|im1k|inno|ipaq|iris|ja(t|v)a|jbro|jemu|jigs|kddi|keji|kgt( |\/)|klon|kpt |kwc\-|kyo(c|k)|le(no|xi)|lg( g|\/(k|l|u)|50|54|\-[a-w])|libw|lynx|m1\-w|m3ga|m50\/|ma(te|ui|xo)|mc(01|21|ca)|m\-cr|me(rc|ri)|mi(o8|oa|ts)|mmef|mo(01|02|bi|de|do|t(\-| |o|v)|zz)|mt(50|p1|v )|mwbp|mywa|n10[0-2]|n20[2-3]|n30(0|2)|n50(0|2|5)|n7(0(0|1)|10)|ne((c|m)\-|on|tf|wf|wg|wt)|nok(6|i)|nzph|o2im|op(ti|wv)|oran|owg1|p800|pan(a|d|t)|pdxg|pg(13|\-([1-8]|c))|phil|pire|pl(ay|uc)|pn\-2|po(ck|rt|se)|prox|psio|pt\-g|qa\-a|qc(07|12|21|32|60|\-[2-7]|i\-)|qtek|r380|r600|raks|rim9|ro(ve|zo)|s55\/|sa(ge|ma|mm|ms|ny|va)|sc(01|h\-|oo|p\-)|sdk\/|se(c(\-|0|1)|47|mc|nd|ri)|sgh\-|shar|sie(\-|m)|sk\-0|sl(45|id)|sm(al|ar|b3|it|t5)|so(ft|ny)|sp(01|h\-|v\-|v )|sy(01|mb)|t2(18|50)|t6(00|10|18)|ta(gt|lk)|tcl\-|tdg\-|tel(i|m)|tim\-|t\-mo|to(pl|sh)|ts(70|m\-|m3|m5)|tx\-9|up(\.b|g1|si)|utst|v400|v750|veri|vi(rg|te)|vk(40|5[0-3]|\-v)|vm40|voda|vulc|vx(52|53|60|61|70|80|81|83|85|98)|w3c(\-| )|webc|whit|wi(g |nc|nw)|wmlb|wonu|x700|yas\-|your|zeto|zte\-/i.test(a.substr(0, 4))) check = true; })(navigator.userAgent || navigator.vendor || (window as any).opera);
        return check;
    })();
    const isChrome = navigator.userAgent.includes("Chrome");

    let statusElement = null;
    if (!hasWebgpu) {
        statusElement = <Alert severity="error">
            Your browser does not support WebGPU, which is required for this test.
            Supported browsers are Google Chrome and Microsoft Edge.
        </Alert>;
    } else if (isMobile) {
        statusElement = <Alert severity="error">
            This test is not supported on mobile browsers.
        </Alert>;
    } else if (!isChrome) {
        statusElement = <Alert severity="warning">
            This test has only been tested on Google Chrome and Microsoft Edge.
            Other browsers may not be able to display the contents properly.
        </Alert>;
    } else {
        statusElement = <Alert severity="success">
            All prerequisites for the test are met.
        </Alert>;
    }

    const [accepted, setAccepted] = useState<boolean>(false);
    const canStart = hasWebgpu && !isMobile && accepted;

    const handleChecked = (e, checked) => {
        setAccepted(checked);
    }

    const handleClick = () => {
        const { demo } = app.state;
        demo.currentPage = "demo1";
        app.setProps({ demo });
    }

    const dataAcquisitionTerms = `
    This test requires the collection of anonymized user data, consisting of 
    timing information, interactions and responses, for the sake of evaluation.
    `;

    return (
        <Container style={{ height: "95%", padding: "2rem" }}>
            <Typography variant='h2'><b>Welcome</b></Typography>

            <Typography variant='body1' marginY={2}>
                Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed
                diam nonumy eirmod tempor invidunt ut labore et dolore magna
                aliquyam erat, sed diam voluptua. At vero eos et accusam et
                justo duo dolores et ea rebum. Stet clita kasd gubergren, no
                sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem
                ipsum dolor sit amet, consetetur sadipscing elitr, sed diam
                nonumy eirmod tempor invidunt ut labore et dolore magna
                aliquyam erat, sed diam voluptua. At vero eos et accusam et
                justo duo dolores et ea rebum. Stet clita kasd gubergren, no
                sea takimata sanctus est Lorem ipsum dolor sit amet.
            </Typography>

            <Typography variant="subtitle1" marginTop={5} marginBottom={1}>
                <b>{dataAcquisitionTerms}</b>
            </Typography>

            <FormGroup>
                <FormControlLabel
                    required
                    control={<Checkbox value={accepted} onChange={handleChecked} />}
                    label="I consent to the collection of anonymized user data."
                />
            </FormGroup>

            <Box marginY={2}>
                {statusElement}
            </Box>

            <Container>
                <Box marginY={2}>
                    <Button
                        variant="contained"
                        onClick={handleClick}
                        fullWidth
                        disabled={!canStart}
                    >
                        Start test
                    </Button>
                </Box>
            </Container>
        </Container>
    );
}

function DemoPage1(app: App) {
    const { results } = app.state.demo;

    const [education, setEducation] = useState<LevelOfEducation>(undefined);
    const [analysisProficiency, setAnalysisProficiency] = useState<number>(undefined);
    const [pcProficiency, setPcProficiency] = useState<number>(undefined);

    const handleEducationChange = (e) => {
        results.education = e.target.value as LevelOfEducation;
        setEducation(e.target.value as LevelOfEducation);
    }

    const handleAnalysisProficiencyChange = (e, proficiency) => {
        setAnalysisProficiency(proficiency);
        switch (proficiency) {
            case 1:
                results.analysisProficiency = "NA";
                break;
            case 2:
                results.analysisProficiency = "Fundamental";
                break;
            case 3:
                results.analysisProficiency = "Novice";
                break;
            case 4:
                results.analysisProficiency = "Intermediate";
                break;
            case 5:
                results.analysisProficiency = "Advanced";
                break;
            case 6:
                results.analysisProficiency = "Expert";
                break;
        }
    }

    const handlePcProficiencyChange = (e, proficiency) => {
        setPcProficiency(proficiency);
        switch (proficiency) {
            case 1:
                results.pcProficiency = "NA";
                break;
            case 2:
                results.pcProficiency = "Fundamental";
                break;
            case 3:
                results.pcProficiency = "Novice";
                break;
            case 4:
                results.pcProficiency = "Intermediate";
                break;
            case 5:
                results.pcProficiency = "Advanced";
                break;
            case 6:
                results.pcProficiency = "Expert";
                break;
        }
    }

    const handleClick = () => {
        const { demo } = app.state;
        demo.currentPage = "demo3";
        app.setProps({ demo });
    }

    const proficiencyLabels: { [index: string]: string } = {
        1: "No experience",
        2: "Fundamental",
        3: "Novice",
        4: "Intermediate",
        5: "Advanced",
        6: "Expert",
    };

    const canContinue = education !== undefined
        && analysisProficiency !== undefined
        && pcProficiency !== undefined;

    return (
        <Container>
            <Typography variant="h4">
                <b>Thank you for your interest in participating in this study.</b>
            </Typography>
            <Typography variant="body1" marginY={1}>
                Before we begin with the interactive test, we would like to ask you
                to fill out the short questionnaire below.
            </Typography>

            <Divider />

            <Typography variant="subtitle1" marginY={2}>
                What is your level of education?
            </Typography>
            <FormControl>
                <InputLabel id="education-label">Level of education</InputLabel>
                <Select
                    labelId="education-label"
                    id="education-select"
                    value={education}
                    label="Education"
                    sx={{ m: 1, minWidth: 240 }}
                    onChange={handleEducationChange}
                >
                    <MenuItem value="Childhood">Early childhood Education</MenuItem>
                    <MenuItem value="Primary">Primary education</MenuItem>
                    <MenuItem value="LowerSecondary">Lower secondary education</MenuItem>
                    <MenuItem value="UpperSecondary">Upper secondary education</MenuItem>
                    <MenuItem value="Post-secondary">Post-secondary non-tertiary education</MenuItem>
                    <MenuItem value="Tertiary">Short-cycle tertiary education</MenuItem>
                    <MenuItem value="Bachelor">Bachelor or equivalent</MenuItem>
                    <MenuItem value="Master">Master or equivalent</MenuItem>
                    <MenuItem value="Doctoral">Doctoral or equivalent</MenuItem>
                </Select>
            </FormControl>

            <Typography variant="subtitle1" marginY={2}>
                How would you describe your proficiency in the task of data analysis?
            </Typography>
            <Box
                margin={1}
                sx={{
                    width: 400,
                    display: 'flex',
                    alignItems: 'center',
                }}
            >
                <Rating
                    name="analysis-proficiency"
                    value={analysisProficiency}
                    max={6}
                    size="large"
                    onChange={handleAnalysisProficiencyChange}
                    emptyIcon={<StarIcon style={{ opacity: 0.55 }} fontSize="inherit" />}
                />
                <Box sx={{ ml: 2 }}>{proficiencyLabels[analysisProficiency]}</Box>
            </Box>



            <Typography variant="subtitle1" marginY={2}>
                How would you describe your proficiency with parallel coordinates?
            </Typography>
            <Box
                margin={1}
                sx={{
                    width: 400,
                    display: 'flex',
                    alignItems: 'center',
                }}
            >
                <Rating
                    name="pc-proficiency"
                    value={analysisProficiency}
                    max={6}
                    size="large"
                    onChange={handlePcProficiencyChange}
                    emptyIcon={<StarIcon style={{ opacity: 0.55 }} fontSize="inherit" />}
                />
                <Box sx={{ ml: 2 }}>{proficiencyLabels[pcProficiency]}</Box>
            </Box>

            <Container>
                <Box marginY={4}>
                    <Button
                        variant="contained"
                        onClick={handleClick}
                        fullWidth
                        disabled={!canContinue}
                    >
                        Next
                    </Button>
                </Box>
            </Container>
        </Container>
    )
}

function DemoPage3(app: App) {
    const {
        ppcState,
        demo,
    } = app.state;
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
                        {InstructionsDialog(demo, app.setProps)}

                        {TaskView(ppcState, demo, app.setProps)}
                        <Divider flexItem />
                        {LabelsView(ppcState, demo, app.setProps)}
                        <Divider flexItem />

                        <div>
                            {AttributeList(ppcState, axes, app.setProps)}
                            {ColorSettings(ppcState, demo, app.setProps)}
                            {ActionsInfo(ppcState)}
                            {DebugInfo(ppcState, demo, app.setProps)}
                        </div>
                    </Stack>
                </Grid>
            </Grid>
        </Box>
    )
}

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
            maxWidth={"xl"}
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
    const interactionMode = ppc.interactionMode ? ppc.interactionMode : InteractionMode.Full;

    let selectionBounds = activeLabel ? labels[activeLabel].selectionBounds : [EPSILON, 1];
    if (!selectionBounds) {
        selectionBounds = [EPSILON, 1];
    }

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
                    }
                }
                setProps({ ppcState: ppc });
            } : null;
        const toggleActive = () => {
            ppc.activeLabel = k;
            setProps({ ppcState: ppc });
        }

        return LabelsViewItem(k === activeLabel, k, deleteLabel, toggleActive);
    });

    const handleProbabilityRangeChange = (e, range) => {
        const labels_clone = window.structuredClone(labels);
        labels_clone[activeLabel].selectionBounds = range as [number, number];
        ppc.labels = labels_clone;
        setProps({ ppcState: ppc })
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
                            value={selectionBounds}
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
    const { userGroup } = demo;
    const { colors, colorBar, axes } = ppc;

    const constantColorModeValue = typeof (ppc.colors?.selected?.color) == "number"
        ? ppc.colors?.selected?.color
        : 0.5;
    const attributeColorModeValue = typeof (ppc.colors?.selected?.color) == "string"
        ? ppc.colors?.selected?.color
        : Object.keys(ppc.axes)[0];

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
        setProps({ ppcState: ppc, demo })
    };

    const setAttributeColorValue = (e, value) => {
        const colors_clone = window.structuredClone(colors);
        colors_clone.selected.color = value;
        ppc.colors = colors_clone;
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

const constructTasks = (userGroup: UserGroup) => {
    return [
        task_1(userGroup),
        task_2(userGroup),
    ];
}

const task_1 = (userGroup: UserGroup): DemoTask => {
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

const task_2 = (userGroup: UserGroup): DemoTask => {
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