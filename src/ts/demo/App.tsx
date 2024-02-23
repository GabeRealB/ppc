/* eslint no-magic-numbers: 0 */
import { S3Client, PutObjectCommand } from '@aws-sdk/client-s3';
import React, { Component, createElement, useEffect, useState } from 'react';
import { v4 as uuid } from 'uuid';
import pako from 'pako';

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
import Paper from '@mui/material/Paper';
import Grid from '@mui/material/Unstable_Grid2';
import Alert from '@mui/material/Alert';
import Checkbox from '@mui/material/Checkbox';
import Select from '@mui/material/Select';
import InputLabel from '@mui/material/InputLabel';
import MenuItem from '@mui/material/MenuItem';
import Rating from '@mui/material/Rating';
import TextField from '@mui/material/TextField';
import CircularProgress from '@mui/material/CircularProgress';

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
import brushingInstr from './resources/brushing_instr.mp4'
import extendAxisInstr from './resources/extend_axis_instr.mp4'
import brushingExtInstr from './resources/brushing_ext_instr.mp4'
import brushFadeoutInstr from './resources/brush_fadeout_instr.mp4'
import interpolationChangeInstr from './resources/interpolation_change_instr.mp4'
import labelsNewInstr from './resources/labels_new_instr.mp4'
import labelsCertaintyInstr from './resources/labels_certainty_instr.mp4'
import labelsCompareInstr from './resources/labels_compare_instr.mp4'
import colorsInstr from './resources/colors_instr.mp4'
import colorsCertaintyInstr from './resources/colors_certainty_instr.mp4'
import colorsOrderInstr from './resources/colors_order_instr.mp4'
import attributesInstr from './resources/attributes_instr.mp4'

import PPC from '../components/PPC';
import { Axis, Props, InteractionMode, Brushes, LabelInfo } from '../types'

import { syntheticDataset, adultDataset, ablationDataset } from './datasets';

const EPSILON = 1.17549435082228750797e-38;
const VERSION = 1;

type DemoTask = {
    name: string,
    shortDescription: string,
    instructions: (() => React.JSX.Element)[],
    taskResultInput?: (props: { task: DemoTask, forceUpdate: () => void }) => React.JSX.Element,
    taskResult?: any,
    viewed: boolean,
    initialState: Props,
    finalState: Props,
    canContinue: (ppc: Props, task?: DemoTask) => boolean,
    disableLabels?: boolean,
    disableAttributes?: boolean,
    disableColors?: boolean,
};

type UserGroup = 'PC' | 'PPC';

type TaskMode = 'Full' | 'Tutorial' | 'Eval'

type DemoPage = 'welcome'
    | 'demo1'
    | 'demo2'
    | 'finish';

type Sex = 'male' | 'female';

type LevelOfEducation = 'Childhood'
    | 'Primary'
    | 'LowerSecondary'
    | 'UpperSecondary'
    | 'Post-secondary'
    | 'Tertiary'
    | 'Bachelor'
    | 'Master'
    | 'Doctoral'

type ColorAbnormality = 'none'
    | 'protanomaly'
    | 'protanopia'
    | 'deuteranomaly'
    | 'deuteranopia'
    | 'tritanomaly'
    | 'tritanopia'
    | 'blue-cone-monochromacy'
    | 'achromatopsia'
    | 'tetrachromacy'

type Proficiency = 'NA'
    | 'Fundamental'
    | 'Novice'
    | 'Intermediate'
    | 'Advanced'
    | 'Expert'

type LogEvent = {
    type: 'start' | 'event' | 'end',
    timestamp: DOMHighResTimeStamp,
    data?: any
};

type TaskLog = {
    events: LogEvent[],
    result?: any
}

type Results = {
    age?: number,
    sex?: Sex,
    education?: LevelOfEducation,
    colorAbnormality?: ColorAbnormality,
    analysisProficiency?: Proficiency,
    pcProficiency?: Proficiency,
    taskLogs: TaskLog[],
};

type DemoState = {
    currentPage: DemoPage,

    userId: uuid,
    userGroup: UserGroup,

    currentTask: number,
    tasks: DemoTask[],
    showInstructions: boolean,

    showDebugInfo: boolean,
    dryRun: boolean,

    deadline: Date,
    deadlinePassed: boolean,

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
        this.logPPCEvent = this.logPPCEvent.bind(this);
        this.forceReRender = this.forceReRender.bind(this);

        const searchParams = new URLSearchParams(window.location.search);
        const debugMode = searchParams.has('debug');
        const dryRun = searchParams.has('dryRun');
        let taskMode = searchParams.get('taskMode');
        if (!['Full', 'Tutorial', 'Eval'].includes(taskMode)) {
            taskMode = 'Full' as TaskMode;
        }
        let userGroup = searchParams.get('userGroup');
        if (userGroup !== 'PC' && userGroup !== 'PPC') {
            userGroup = Math.random() < 0.5 ? 'PC' : 'PPC';
        }

        const deadline = new Date(2024, 2, 31);
        const deadlinePassed = Date.now() > deadline.getTime();

        const tasks = constructTasks(userGroup as UserGroup, taskMode as TaskMode);
        const ppc = window.structuredClone(tasks[0].initialState);
        ppc.setProps = this.setPPCProps;

        const taskLogs = tasks.map(() => ({ events: [] } as TaskLog))

        this.state = {
            ppcState: ppc,
            demo: {
                currentPage: 'welcome',
                userId: uuid(),
                userGroup: userGroup as UserGroup,
                showInstructions: true,
                currentTask: 0,
                tasks: tasks,
                showDebugInfo: debugMode,
                dryRun: dryRun,
                deadline,
                deadlinePassed,
                results: { taskLogs }
            },
        };
    }

    setProps(newProps) {
        this.setState(newProps);
    }

    setPPCProps(newProps) {
        const { ppcState } = this.state;

        this.logPPCEvent(newProps);
        for (const [k, v] of Object.entries(newProps)) {
            ppcState[k] = v;
        }

        this.setProps({ ppcState });
    }

    logPPCEvent(newProps) {
        const { demo } = this.state;
        const { results, currentTask } = demo;
        const { taskLogs } = results;
        const log = taskLogs[currentTask];

        const data = window.structuredClone(newProps);
        if ('selectionProbabilities' in data) {
            delete data['selectionProbabilities'];
        }
        if ('selectionIndices' in data) {
            delete data['selectionIndices'];
        }
        if (Object.keys(data).length == 0) {
            return;
        }

        const event: LogEvent = {
            type: 'event',
            timestamp: performance.now(),
            data,
        };
        log.events.push(event);
    }

    forceReRender() {
        this.forceUpdate();
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
            case 'demo2':
                page = createElement(DemoPage2, this);
                break;
            case 'finish':
                page = createElement(FinishPage, this);
                break;
        }

        return page;
    }
}

export default App;

function WelcomePage(app: App) {
    const [webgpuTestStatus, setWebgpuTestStatus] = useState<boolean>(undefined);
    useEffect(() => {
        (async () => {
            if (!navigator.gpu) {
                setWebgpuTestStatus(false);
                return;
            }

            const gpu = navigator.gpu;
            const adapter = await gpu.requestAdapter();
            if (!adapter) {
                setWebgpuTestStatus(false);
                return;
            }

            const device = await adapter.requestDevice();
            if (!device) {
                setWebgpuTestStatus(false);
                return;
            }
            device.destroy();
            setWebgpuTestStatus(true);
        })();
    }, [])

    const isMobile = (() => {
        let check = false;
        (function (a) { if (/(android|bb\d+|meego).+mobile|avantgo|bada\/|blackberry|blazer|compal|elaine|fennec|hiptop|iemobile|ip(hone|od)|iris|kindle|lge |maemo|midp|mmp|mobile.+firefox|netfront|opera m(ob|in)i|palm( os)?|phone|p(ixi|re)\/|plucker|pocket|psp|series(4|6)0|symbian|treo|up\.(browser|link)|vodafone|wap|windows ce|xda|xiino|android|ipad|playbook|silk/i.test(a) || /1207|6310|6590|3gso|4thp|50[1-6]i|770s|802s|a wa|abac|ac(er|oo|s\-)|ai(ko|rn)|al(av|ca|co)|amoi|an(ex|ny|yw)|aptu|ar(ch|go)|as(te|us)|attw|au(di|\-m|r |s )|avan|be(ck|ll|nq)|bi(lb|rd)|bl(ac|az)|br(e|v)w|bumb|bw\-(n|u)|c55\/|capi|ccwa|cdm\-|cell|chtm|cldc|cmd\-|co(mp|nd)|craw|da(it|ll|ng)|dbte|dc\-s|devi|dica|dmob|do(c|p)o|ds(12|\-d)|el(49|ai)|em(l2|ul)|er(ic|k0)|esl8|ez([4-7]0|os|wa|ze)|fetc|fly(\-|_)|g1 u|g560|gene|gf\-5|g\-mo|go(\.w|od)|gr(ad|un)|haie|hcit|hd\-(m|p|t)|hei\-|hi(pt|ta)|hp( i|ip)|hs\-c|ht(c(\-| |_|a|g|p|s|t)|tp)|hu(aw|tc)|i\-(20|go|ma)|i230|iac( |\-|\/)|ibro|idea|ig01|ikom|im1k|inno|ipaq|iris|ja(t|v)a|jbro|jemu|jigs|kddi|keji|kgt( |\/)|klon|kpt |kwc\-|kyo(c|k)|le(no|xi)|lg( g|\/(k|l|u)|50|54|\-[a-w])|libw|lynx|m1\-w|m3ga|m50\/|ma(te|ui|xo)|mc(01|21|ca)|m\-cr|me(rc|ri)|mi(o8|oa|ts)|mmef|mo(01|02|bi|de|do|t(\-| |o|v)|zz)|mt(50|p1|v )|mwbp|mywa|n10[0-2]|n20[2-3]|n30(0|2)|n50(0|2|5)|n7(0(0|1)|10)|ne((c|m)\-|on|tf|wf|wg|wt)|nok(6|i)|nzph|o2im|op(ti|wv)|oran|owg1|p800|pan(a|d|t)|pdxg|pg(13|\-([1-8]|c))|phil|pire|pl(ay|uc)|pn\-2|po(ck|rt|se)|prox|psio|pt\-g|qa\-a|qc(07|12|21|32|60|\-[2-7]|i\-)|qtek|r380|r600|raks|rim9|ro(ve|zo)|s55\/|sa(ge|ma|mm|ms|ny|va)|sc(01|h\-|oo|p\-)|sdk\/|se(c(\-|0|1)|47|mc|nd|ri)|sgh\-|shar|sie(\-|m)|sk\-0|sl(45|id)|sm(al|ar|b3|it|t5)|so(ft|ny)|sp(01|h\-|v\-|v )|sy(01|mb)|t2(18|50)|t6(00|10|18)|ta(gt|lk)|tcl\-|tdg\-|tel(i|m)|tim\-|t\-mo|to(pl|sh)|ts(70|m\-|m3|m5)|tx\-9|up(\.b|g1|si)|utst|v400|v750|veri|vi(rg|te)|vk(40|5[0-3]|\-v)|vm40|voda|vulc|vx(52|53|60|61|70|80|81|83|85|98)|w3c(\-| )|webc|whit|wi(g |nc|nw)|wmlb|wonu|x700|yas\-|your|zeto|zte\-/i.test(a.substr(0, 4))) check = true; })(navigator.userAgent || navigator.vendor || (window as any).opera);
        return check;
    })();
    const isChrome = navigator.userAgent.includes('Chrome');
    const isLinux = /(Linux|X11(?!.*CrOS))/.test(navigator.userAgent);

    let statusElement = null;
    if (webgpuTestStatus === false) {
        statusElement = <Alert severity='error'>
            Your browser does not support WebGPU, which is required for this test.
            Supported browsers are Google Chrome and Microsoft Edge.
        </Alert>;
    } else if (isMobile) {
        statusElement = <Alert severity='error'>
            This test is not supported on mobile browsers.
        </Alert>;
    } else if (!isChrome) {
        statusElement = <Alert severity='warning'>
            This test has only been tested on Google Chrome and Microsoft Edge.
            Other browsers may not be able to display the contents properly.
        </Alert>;
    } else if (isLinux) {
        statusElement = <Alert severity='warning'>
            This website has detected that you are running a Linux variant as your
            operating system. As of now, WebGPU support is marked as experimental
            on most browsers running in Linux, and may need to be enabled manually.
            As a result, your browser may not be able to display the contents properly.
        </Alert>;
    } else {
        statusElement = <Alert severity='success'>
            All prerequisites for the test are met.
        </Alert>;
    }

    const [accepted, setAccepted] = useState<boolean>(false);
    const canStart = webgpuTestStatus && !isMobile && accepted;

    const handleChecked = (e, checked) => {
        setAccepted(checked);
    }

    const handleClick = () => {
        const { demo } = app.state;
        demo.currentPage = 'demo1';
        demo.results.taskLogs[0].events.push({
            type: 'start',
            timestamp: performance.now(),
        });
        app.setProps({ demo });
    }

    const dataAcquisitionTerms = `
    This test requires the collection of anonymized user data, consisting of 
    timing information, interactions and responses, for the sake of evaluation.
    `;

    const { deadline, deadlinePassed, dryRun } = app.state.demo;

    return (
        <Container style={{ height: '95%', padding: '2rem' }}>
            <Typography variant='h2'><b>Welcome</b></Typography>

            <Typography variant='body1' marginY={2}>
                The analysis of multidimensional datasets is an important
                aspect in the fields of visualization and data analysis.
                One way to tackle such tasks is to perform an interactive
                visual analysis, by visualizing the datasets using parallel
                coordinates. Parallel coordinates is a plot, where each
                dimension, or attribute, is represented as individual axes,
                and the multidimensional points as curves connecting those axes.
                In practice, we must also contend with some degree of uncertainty,
                which is contained in the dataset. This uncertainty may be caused
                by measurement errors, variance in the data, et cetera. In this
                study, we want to measure the effectiveness of extending the
                standard parallel coordinates plot with utilities that allow one
                to model the certainty in the data.
            </Typography>

            <Typography variant='subtitle1' marginTop={5} marginBottom={1}>
                <b>{dataAcquisitionTerms}</b>
            </Typography>

            <FormGroup>
                <FormControlLabel
                    required
                    control={<Checkbox value={accepted} onChange={handleChecked} />}
                    label='I consent to the collection of anonymized user data.'
                />
            </FormGroup>

            <Box marginY={2}>
                {statusElement}
            </Box>

            {deadlinePassed && !dryRun ?
                <Box marginY={2}>
                    <Alert severity='warning'>
                        The study ran until {deadline.toLocaleDateString(undefined, {
                            weekday: 'long',
                            year: 'numeric',
                            month: 'long',
                            day: 'numeric',
                        })}.
                        You can still take the test, but the results will not be
                        used for the evaluation.
                    </Alert>
                </Box> : undefined}

            {dryRun ?
                <Box marginY={2}>
                    <Alert severity='info'>
                        Dry run mode is active, the results will not be used for
                        the evaluation.
                    </Alert>
                </Box> : undefined}

            <Container>
                <Box marginY={2}>
                    <Button
                        variant='contained'
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

    const [age, setAge] = useState<number>(undefined);
    const [sex, setSex] = useState<Sex>(undefined);
    const [education, setEducation] = useState<LevelOfEducation>(undefined);
    const [colorAbnormalty, setAbnormality] = useState<ColorAbnormality>(undefined);
    const [analysisProficiency, setAnalysisProficiency] = useState<number>(0);
    const [pcProficiency, setPcProficiency] = useState<number>(0);

    const handleAgeChange = (e) => {
        var age = parseInt(e.target.value);
        if (age < 0) {
            age = 0;
        }
        results.age = age;
        setAge(age);
    }

    const handleSexChange = (e) => {
        results.sex = e.target.value as Sex;
        setSex(e.target.value as Sex);
    }

    const handleEducationChange = (e) => {
        results.education = e.target.value as LevelOfEducation;
        setEducation(e.target.value as LevelOfEducation);
    }

    const handleColorAbnormalityChange = (e) => {
        results.colorAbnormality = e.target.value as ColorAbnormality;
        setAbnormality(e.target.value as ColorAbnormality);
    }

    const handleAnalysisProficiencyChange = (e, proficiency) => {
        proficiency = proficiency ? proficiency : 0;
        setAnalysisProficiency(proficiency);
        switch (proficiency) {
            case 1:
                results.analysisProficiency = 'NA';
                break;
            case 2:
                results.analysisProficiency = 'Fundamental';
                break;
            case 3:
                results.analysisProficiency = 'Novice';
                break;
            case 4:
                results.analysisProficiency = 'Intermediate';
                break;
            case 5:
                results.analysisProficiency = 'Advanced';
                break;
            case 6:
                results.analysisProficiency = 'Expert';
                break;
        }
    }

    const handlePcProficiencyChange = (e, proficiency) => {
        proficiency = proficiency ? proficiency : 0;
        setPcProficiency(proficiency);
        switch (proficiency) {
            case 1:
                results.pcProficiency = 'NA';
                break;
            case 2:
                results.pcProficiency = 'Fundamental';
                break;
            case 3:
                results.pcProficiency = 'Novice';
                break;
            case 4:
                results.pcProficiency = 'Intermediate';
                break;
            case 5:
                results.pcProficiency = 'Advanced';
                break;
            case 6:
                results.pcProficiency = 'Expert';
                break;
        }
    }

    const handleClick = () => {
        const { demo } = app.state;
        demo.currentPage = 'demo2';
        app.setProps({ demo });
    }

    const proficiencyLabels: { [index: string]: string } = {
        1: 'No experience',
        2: 'Fundamental',
        3: 'Novice',
        4: 'Intermediate',
        5: 'Advanced',
        6: 'Expert',
    };

    const ageError = age && age > 120;

    const canContinue = age !== undefined
        && !ageError
        && sex !== undefined
        && education !== undefined
        && colorAbnormalty !== undefined
        && analysisProficiency !== 0
        && pcProficiency !== 0;

    return (
        <Container>
            <Typography variant='h4'>
                <b>Thank you for your interest in participating in this study.</b>
            </Typography>
            <Typography variant='body1' marginY={1}>
                Before we begin with the interactive test, we would like to ask you
                to fill out the short questionnaire below.
            </Typography>

            <Divider />

            <Typography variant='subtitle1' marginY={2}>
                Personal info.
            </Typography>
            <Box margin={1} marginY={1.5}>
                <FormControl fullWidth>
                    <FormLabel id='sex-input-label'>Sex</FormLabel>
                    <RadioGroup
                        row
                        aria-labelledby='sex-input-label'
                        name='sex-input'
                        value={sex}
                        onChange={handleSexChange}
                    >
                        <FormControlLabel value='male' control={<Radio />} label='Male' />
                        <FormControlLabel value='female' control={<Radio />} label='Female' />
                    </RadioGroup>
                </FormControl>
            </Box>
            <Box marginY={1.5}>
                <TextField
                    id="age-input"
                    label="Age"
                    type="number"
                    value={age}
                    error={ageError}
                    helperText={ageError ? 'Age is bigger than 120' : null}
                    sx={{ m: 1, minWidth: 240 }}
                    onChange={handleAgeChange}
                />
            </Box>
            <Box marginY={1.5}>
                <FormControl>
                    <InputLabel id='education-label'>Level of education</InputLabel>
                    <Select
                        labelId='education-label'
                        id='education-select'
                        value={education}
                        label='Education'
                        sx={{ m: 1, minWidth: 240 }}
                        onChange={handleEducationChange}
                    >
                        <MenuItem value='Childhood'>Early childhood Education</MenuItem>
                        <MenuItem value='Primary'>Primary education</MenuItem>
                        <MenuItem value='LowerSecondary'>Lower secondary education</MenuItem>
                        <MenuItem value='UpperSecondary'>Upper secondary education</MenuItem>
                        <MenuItem value='Post-secondary'>Post-secondary non-tertiary education</MenuItem>
                        <MenuItem value='Tertiary'>Short-cycle tertiary education</MenuItem>
                        <MenuItem value='Bachelor'>Bachelor or equivalent</MenuItem>
                        <MenuItem value='Master'>Master or equivalent</MenuItem>
                        <MenuItem value='Doctoral'>Doctoral or equivalent</MenuItem>
                    </Select>
                </FormControl>
            </Box>
            <Box marginY={1.5}>
                <FormControl>
                    <InputLabel id='color-abnormalities-label'>Color vision abnormality</InputLabel>
                    <Select
                        labelId='color-abnormalities-label'
                        id='color-abnormalities-select'
                        value={colorAbnormalty}
                        label='Vision abnormality'
                        sx={{ m: 1, minWidth: 240 }}
                        onChange={handleColorAbnormalityChange}
                    >
                        <MenuItem value='none'>No abnormality</MenuItem>
                        <MenuItem value='protanomaly'>Protanomaly</MenuItem>
                        <MenuItem value='protanopia'>Protanopia</MenuItem>
                        <MenuItem value='deuteranomaly'>Deuteranomaly</MenuItem>
                        <MenuItem value='deuteranopia'>Deuteranopia</MenuItem>
                        <MenuItem value='tritanomaly'>Tritanomaly</MenuItem>
                        <MenuItem value='tritanopia'>Tritanopia</MenuItem>
                        <MenuItem value='blue-cone-monochromacy'>Blue cone monochromacy</MenuItem>
                        <MenuItem value='achromatopsia'>Achromatopsia</MenuItem>
                        <MenuItem value='tetrachromacy'>Tetrachromacy</MenuItem>
                    </Select>
                </FormControl>
            </Box>

            <Typography variant='subtitle1' marginY={2}>
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
                    name='analysis-proficiency'
                    value={analysisProficiency}
                    max={6}
                    size='large'
                    onChange={handleAnalysisProficiencyChange}
                    emptyIcon={<StarIcon style={{ opacity: 0.55 }} fontSize='inherit' />}
                />
                <Box sx={{ ml: 2 }}>{proficiencyLabels[analysisProficiency]}</Box>
            </Box>

            <Typography variant='subtitle1' marginY={2}>
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
                    name='pc-proficiency'
                    value={pcProficiency}
                    max={6}
                    size='large'
                    onChange={handlePcProficiencyChange}
                    emptyIcon={<StarIcon style={{ opacity: 0.55 }} fontSize='inherit' />}
                />
                <Box sx={{ ml: 2 }}>{proficiencyLabels[pcProficiency]}</Box>
            </Box>

            <Container>
                <Box marginY={4}>
                    <Button
                        variant='contained'
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

function DemoPage2(app: App) {
    const {
        ppcState,
        demo,
    } = app.state;
    const {
        axes,
    } = ppcState;

    return (
        <Box style={{ height: '95%', padding: '2rem' }}>
            <Grid container style={{ height: '100%' }} spacing={2}>
                <Grid xs={10}>
                    <PPC
                        {...ppcState}
                    />
                </Grid>
                <Grid xs={2} maxHeight={'95%'} sx={{ overflow: 'auto' }}>
                    <Stack
                        spacing={2}
                        justifyContent='flex-start'
                        alignItems='flex-start'
                        paddingX={'2rem'}
                    >
                        <InstructionsDialog demo={demo} setProps={app.setProps} />

                        {TaskView(ppcState, demo, app.setProps, app.logPPCEvent, app.forceReRender)}
                        <Divider flexItem />
                        {LabelsView(ppcState, demo, app.setProps, app.logPPCEvent)}
                        <Divider flexItem />

                        <div>
                            {AttributeList(ppcState, demo, axes, app.setProps, app.logPPCEvent)}
                            {ColorSettings(ppcState, demo, app.setProps, app.logPPCEvent)}
                            {ActionsInfo(ppcState)}
                            {DebugInfo(ppcState, demo, app.setProps, app.logPPCEvent)}
                        </div>
                    </Stack>
                </Grid>
            </Grid>
        </Box>
    )
}

function FinishPage(app: App) {
    const { demo } = app.state;
    const { results, userId, userGroup, deadlinePassed, dryRun } = demo;

    const [finished, setFinished] = useState<{ error: any } | boolean>(deadlinePassed);

    useEffect(() => {
        if (finished) {
            return;
        }

        const fileName = `${uuid()}.bin`;
        const fileContents = { userId, userGroup, results, VERSION };
        const fileContentsJSON = JSON.stringify(fileContents);
        const fileContentsCompressed = pako.deflate(fileContentsJSON);

        if (dryRun) {
            console.info('Dry run results', fileName, fileContents, fileContentsCompressed);
            console.info('Raw length', fileContentsJSON.length);
            console.info('Compressed length', fileContentsCompressed.length);
            setFinished(true);
            return;
        }

        const client = new S3Client({
            region: 'eu-central-1',
            endpoint: 'https://s3.hidrive.strato.com',
            credentials: {
                accessKeyId: 'AHS4MOAD6Q7YF20RJLZX',
                secretAccessKey: 'IE0g/gBAFSix49pPMwZAcnWwe7OhtWWj9/ACeHQY',
            }
        });
        const command = new PutObjectCommand({
            Bucket: 'userstudy',
            Key: fileName,
            Body: fileContentsCompressed,
        });

        client.send(command).then(() => {
            setFinished(true);
        }).catch((e) => {
            setFinished({ error: e });
        })
    }, []);

    return (
        <Container>
            {!finished ?
                <Box sx={{
                    position: 'absolute',
                    top: '50%',
                    left: '50%',
                    marginTop: '-50px',
                    marginLeft: '-50px',
                    width: '100px',
                    height: '100px'
                }}>
                    <CircularProgress />
                </Box> : undefined
            }
            {
                finished && typeof (finished) === 'boolean' ?
                    <>
                        <Typography variant='h4'>
                            <b>Thank you for your participation in this study.</b>
                        </Typography>
                        <Typography variant='body1' marginY={2}>
                            You may now close this page.
                        </Typography>
                    </> : undefined
            }
            {
                finished && typeof (finished) == 'object' ?
                    <Alert severity='error'>
                        Could not submit results. Error: {finished.error}.
                    </Alert> : undefined}
        </Container >
    )
}

const InstructionsDialog = (props: {
    demo: DemoState;
    setProps: (newProps) => void
}) => {
    const { demo, setProps } = props;
    const { showInstructions, currentTask, tasks } = demo;
    const task = tasks[currentTask];
    const { name } = task;

    const [pageIdx, setPageIdx] = useState<number>(0);

    const handleNext = () => {
        if (pageIdx == task.instructions.length - 1) {
            handleClose();
        } else {
            setPageIdx(pageIdx + 1);
        }
    }

    const handlePrevious = () => {
        setPageIdx(pageIdx - 1);
    }

    const handleClose = () => {
        demo.showInstructions = false;
        setPageIdx(0);
        setProps({ demo });
    };

    useEffect(() => {
        const video = document.getElementById('instructions_video') as HTMLMediaElement;
        if (video) {
            video.load();
        }
    });

    return (
        <Dialog
            onClose={handleClose}
            open={showInstructions}
            maxWidth={'xl'}
            scroll='paper'
            aria-labelledby='instructions-dialog-title'
            aria-describedby='instructions-dialog-description'
        >
            <DialogTitle id='instructions-dialog-title'>
                {name}
            </DialogTitle>
            <DialogContent id='instructions-dialog-description' dividers>
                {task.instructions[pageIdx]()}
            </DialogContent>

            {task.instructions.length === 1 ? <DialogActions>
                <Button onClick={handleClose} autoFocus>
                    Close
                </Button>
            </DialogActions> : undefined}
            {task.instructions.length !== 1 ? <MobileStepper
                variant='dots'
                steps={task.instructions.length}
                position='static'
                activeStep={pageIdx}
                nextButton={
                    <Button size='small' onClick={handleNext}>
                        {pageIdx !== task.instructions.length - 1 ? 'Next' : 'Close'}
                        <KeyboardArrowRight />
                    </Button>
                }
                backButton={
                    <Button size='small' onClick={handlePrevious} disabled={pageIdx === 0}>
                        <KeyboardArrowLeft />
                        Previous
                    </Button>
                }
            /> : undefined}
        </Dialog>
    )
}

const TaskView = (
    ppc: Props,
    demo: DemoState,
    setProps: (newProps) => void,
    logPPCEvent: (newProps) => void,
    forceUpdate: () => void,
) => {
    const { currentTask, tasks, results } = demo;
    const task = tasks[currentTask];
    const { name, shortDescription } = task;

    const handleNext = () => {
        const { taskLogs } = results;
        const timestamp = performance.now();

        const current = tasks[currentTask];
        const currentLog = taskLogs[currentTask];
        currentLog.events.push({
            type: 'end',
            timestamp
        });

        if (current.taskResult) {
            currentLog.result = window.structuredClone(current.taskResult);
        }

        const setPpcProps = ppc.setProps;
        ppc.setProps = undefined;
        current.finalState = window.structuredClone(ppc);

        if (tasks.length - 1 != currentTask) {
            const next = tasks[currentTask + 1];
            const nextLog = taskLogs[currentTask + 1];

            nextLog.events.push({
                type: 'start',
                timestamp
            });

            let nextPpc = window.structuredClone(next.finalState);
            if (!nextPpc) {
                nextPpc = window.structuredClone(next.initialState);
            }
            nextPpc.setProps = setPpcProps;

            demo.currentTask = currentTask + 1;
            demo.showInstructions = !next.viewed;
            setProps({ ppcState: nextPpc, demo });
        } else {
            demo.currentPage = 'finish';
            setProps({ demo });
        }
    };

    const handleBack = () => {
        const { taskLogs } = results;
        const timestamp = performance.now();

        const current = tasks[currentTask];
        const currentLog = taskLogs[currentTask];
        currentLog.events.push({
            type: 'end',
            timestamp
        });

        const prev = tasks[currentTask - 1];
        const prevLog = taskLogs[currentTask - 1];
        prevLog.events.push({
            type: 'start',
            timestamp
        });

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
        current.taskResult = undefined;

        const newPPC = window.structuredClone(current.initialState);
        newPPC.setProps = ppc.setProps;
        logPPCEvent(current.initialState);
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
        <Stack width={'100%'} spacing={1}>
            <Typography variant='h5'>{name}</Typography>
            <Typography variant='subtitle1'>{shortDescription}</Typography>
            <Container>
                <Button
                    variant='contained'
                    startIcon={<HelpIcon />}
                    sx={{ width: '95%' }}
                    onClick={openInstructionsDialog}
                >
                    Instructions
                </Button>
            </Container>
            <Container>
                <Button
                    variant='contained'
                    startIcon={<RestartAltIcon />}
                    sx={{ width: '95%' }}
                    onClick={handleReset}
                >
                    Reset
                </Button>
            </Container>
            {task.taskResultInput
                ? createElement(task.taskResultInput, { task, forceUpdate })
                : undefined}
            <MobileStepper
                variant='progress'
                steps={tasks.length}
                position='bottom'
                activeStep={currentTask}
                nextButton={
                    <Button size='small' onClick={handleNext} disabled={!task.canContinue(ppc, task)}>
                        {currentTask !== tasks.length - 1 ? 'Next' : 'Finish'}
                        <KeyboardArrowRight />
                    </Button>
                }
                backButton={
                    <Button size='small' onClick={handleBack} disabled={currentTask === 0}>
                        <KeyboardArrowLeft />
                        Previous
                    </Button>
                }
            />
        </Stack>
    )
}

const LabelsViewItem = (
    active: boolean,
    name: string,
    deleteLabel: () => void,
    toggleActive: () => void
) => {
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
            <Divider sx={{ height: 28, m: 0.5 }} orientation='vertical' />
            <IconButton onClick={deleteLabel} disabled={deleteLabel == null}>
                <DeleteIcon />
            </IconButton>
        </Paper>
    )
}

const LabelsView = (
    ppc: Props,
    demo: DemoState,
    setProps: (newProps) => void,
    logPPCEvent: (newProps) => void
) => {
    const { tasks, currentTask } = demo;
    const { labels, activeLabel } = ppc;
    const interactionMode = ppc.interactionMode ? ppc.interactionMode : InteractionMode.Full;

    if (tasks[currentTask].disableLabels) {
        return (undefined);
    }

    let selectionBounds = activeLabel ? labels[activeLabel].selectionBounds : [EPSILON, 1];
    if (!selectionBounds) {
        selectionBounds = [EPSILON, 1];
    }

    const items = Object.entries(labels).map(([k, v]) => {
        const deleteLabel = interactionMode == InteractionMode.Compatibility
            || interactionMode == InteractionMode.Full
            ? () => {
                const labelsClone = window.structuredClone(labels);
                delete labelsClone[k];
                ppc.labels = labelsClone;
                if (activeLabel === k) {
                    const keys = Object.keys(labelsClone);
                    if (keys.length == 0) {
                        ppc.activeLabel = null;
                    } else {
                        ppc.activeLabel = keys[keys.length - 1];
                    }
                    logPPCEvent({ labels: labelsClone, activeLabel: ppc.activeLabel });
                } else {
                    logPPCEvent({ labels: labelsClone });
                }
                setProps({ ppcState: ppc });
            } : null;
        const toggleActive = () => {
            ppc.activeLabel = k;
            logPPCEvent({ activeLabel: k });
            setProps({ ppcState: ppc });
        }

        return LabelsViewItem(k === activeLabel, k, deleteLabel, toggleActive);
    });

    const [labelName, setLabelName] = useState<string>('');
    const canAddLabel = labelName !== '' && !Object.keys(labels).includes(labelName);

    const handleLabelNameChanged = (e) => {
        setLabelName(e.target.value);
    };

    const createNewLabel = (e) => {
        const labelsClone = window.structuredClone(labels);
        const newActiveLabel = labelName;
        labelsClone[labelName] = {};
        ppc.labels = labelsClone;
        ppc.activeLabel = newActiveLabel;
        logPPCEvent({ labels: labelsClone, activeLabel: newActiveLabel });
        setProps({ ppcState: ppc });
        setLabelName('');
    };

    const handleProbabilityRangeChange = (e: Event, range: Array<number>) => {
        const labelsClone = window.structuredClone(labels);
        labelsClone[activeLabel].selectionBounds = range as [number, number];
        ppc.labels = labelsClone;

        if (e.type === 'mouseup') {
            logPPCEvent({ labels: labelsClone });
        }
        setProps({ ppcState: ppc })
    };

    const handleEnter = (e: React.KeyboardEvent<HTMLInputElement | HTMLTextAreaElement>) => {
        if (e.key === 'Enter' && canAddLabel) {
            createNewLabel(e);
            e.preventDefault();
        }
    }

    const probabilityLabelFormat = (value: number) => {
        let percent = Math.round((value + Number.EPSILON) * 100 * 100) / 100;
        return `${percent}%`
    };

    return (
        <Box width={'100%'}>
            <Typography variant='h5'>Labels</Typography>
            <Stack spacing={1}>
                <div />
                {items}

                {interactionMode == InteractionMode.Compatibility
                    || interactionMode == InteractionMode.Full
                    ? <Paper
                        component='form'
                        sx={{ display: 'flex', alignItems: 'center' }}
                    >
                        <InputBase
                            sx={{ ml: 1, flex: 1 }}
                            placeholder='New label name'
                            value={labelName}
                            onChange={handleLabelNameChanged}
                            onKeyDown={handleEnter}
                        />
                        <IconButton
                            type='button'
                            sx={{ p: '10px' }}
                            onClick={createNewLabel}
                            disabled={!canAddLabel}
                        >
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
                            onChangeCommitted={handleProbabilityRangeChange}
                            getAriaValueText={probabilityLabelFormat}
                            valueLabelFormat={probabilityLabelFormat}
                            valueLabelDisplay='auto'
                            size='small'
                        />
                    </FormControl> : undefined}
            </Stack>
        </Box>
    )
}

const AttributeListItem = (
    visible: boolean,
    axis: Axis,
    handleClick: () => void
) => {
    const { label } = axis;
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
            <Divider sx={{ height: 28, m: 0.5 }} orientation='vertical' />
            <IconButton onClick={handleClick} disabled={handleClick == null}>
                {visible ? <VisibilityIcon /> : <VisibilityOffIcon />}
            </IconButton>
        </Paper>
    );
}

const AttributeList = (
    ppcState: Props,
    demo: DemoState,
    axes: { [id: string]: Axis },
    setProps: (newProps) => void,
    logPPCEvent: (newProps) => void
) => {
    const { order } = ppcState;
    const { tasks, currentTask } = demo;
    const interactionMode = ppcState.interactionMode ? ppcState.interactionMode : InteractionMode.Full;

    if (tasks[currentTask].disableAttributes) {
        return (undefined);
    }

    const items = [];
    for (const key in axes) {
        const orderIndex = order ? order.indexOf(key) : -1;
        const visible = orderIndex !== -1;

        const update = interactionMode == InteractionMode.Compatibility
            || interactionMode == InteractionMode.Full
            ? () => {
                const newOrder = order ? window.structuredClone(order) : [];
                if (visible) {
                    newOrder.splice(orderIndex, 1);
                } else {
                    newOrder.push(key);
                }
                ppcState.order = newOrder;

                logPPCEvent({ order: newOrder });
                setProps({ ppcState });
            } : null;
        const axis = axes[key];
        items.push(AttributeListItem(visible, axis, update));
    }

    return (
        <Accordion sx={{ width: '100%' }}>
            <AccordionSummary
                id='attributes-header'
                aria-controls='attributes-content'
                expandIcon={<ExpandMoreIcon />}
            >
                <Typography variant='h5'>Attributes</Typography>
            </AccordionSummary>
            <AccordionDetails>
                <Stack width={'100%'} spacing={1}>
                    {items}
                </Stack>
            </AccordionDetails>
        </Accordion>
    );
}

const ColorSettings = (
    ppc: Props,
    demo: DemoState,
    setProps: (newProps) => void,
    logPPCEvent: (newProps) => void
) => {
    const { userGroup, tasks, currentTask } = demo;
    const { colors, colorBar, axes } = ppc;

    if (tasks[currentTask].disableColors) {
        return (undefined);
    }

    const drawOrder = colors.drawOrder ? colors.drawOrder : 'selected_increasing';
    const constantColorModeValue = typeof (ppc.colors?.selected?.color) == 'number'
        ? ppc.colors?.selected?.color
        : 0.5;
    const attributeColorModeValue = typeof (ppc.colors?.selected?.color) == 'string'
        ? ppc.colors?.selected?.color
        : Object.keys(ppc.axes)[0];

    let colorMode;
    if (colors) {
        switch (typeof (colors.selected.color)) {
            case 'number':
                colorMode = 'constant';
                break;
            case 'string':
                colorMode = 'attribute';
            default:
                if (typeof (colors.selected.color) == 'object'
                    && colors.selected.color.type === 'probability') {
                    colorMode = 'probability';
                }
        }
    } else {
        colorMode = 'costant';
    }
    const colorMapValue = colors ? colors.selected.scale as string : null;

    const switchColorBarVisibility = (e, visibility) => {
        ppc.colorBar = visibility;
        logPPCEvent({ colorBar: visibility });
        setProps({ ppcState: ppc })
    };

    const switchColorMode = (e, colorMode) => {
        const colorsClone = colors ?
            window.structuredClone(colors)
            : { selected: { color: 0.5, scale: 'plasma' } };
        switch (colorMode) {
            case 'constant':
                colorsClone.selected.color = constantColorModeValue;
                break;
            case 'attribute':
                colorsClone.selected.color = attributeColorModeValue;
                break;
            case 'probability':
                colorsClone.selected.color = { type: 'probability' };
                break;
        }
        ppc.colors = colorsClone;
        logPPCEvent({ colors: colorsClone });
        setProps({ ppcState: ppc });
    };

    const switchDrawOrder = (e, drawOrder) => {
        const colorsClone = colors ?
            window.structuredClone(colors)
            : { selected: { color: 0.5, scale: 'plasma' } };
        colorsClone.drawOrder = drawOrder;
        ppc.colors = colorsClone;
        logPPCEvent({ colors: colorsClone });
        setProps({ ppcState: ppc });
    }

    const setConstantColorValue = (e: Event, value: number) => {
        const colorsClone = colors ?
            window.structuredClone(colors)
            : { selected: { color: 0.5, scale: 'plasma' } };
        colorsClone.selected.color = value;
        ppc.colors = colorsClone;

        if (e.type === 'mouseup') {
            logPPCEvent({ colors: colorsClone });
        }
        setProps({ ppcState: ppc, demo })
    };

    const setAttributeColorValue = (e, value) => {
        const colorsClone = colors ?
            window.structuredClone(colors)
            : { selected: { color: 0.5, scale: 'plasma' } };
        colorsClone.selected.color = value;
        ppc.colors = colorsClone;
        logPPCEvent({ colors: colorsClone });
        setProps({ ppcState: ppc, demo })
    };

    const setColorMap = (e, colorMap) => {
        const colorsClone = colors ?
            window.structuredClone(colors)
            : { selected: { color: 0.5, scale: 'plasma' } };
        colorsClone.selected.scale = colorMap;
        ppc.colors = colorsClone;
        logPPCEvent({ colors: colorsClone });
        setProps({ ppcState: ppc })
    };

    return (
        <Accordion sx={{ width: '100%' }}>
            <AccordionSummary
                id='color-settings-header'
                aria-controls='color-settings-content'
                expandIcon={<ExpandMoreIcon />}
            >
                <Typography variant='h5'>Colors</Typography>
            </AccordionSummary>
            <AccordionDetails>
                <FormControl fullWidth>
                    <FormLabel id='color-settings-color-bar-group-label'>Color Bar</FormLabel>
                    <RadioGroup
                        row
                        aria-labelledby='color-settings-color-bar-group-label'
                        name='color-settings-color-bar-group'
                        value={colorBar ? colorBar : 'hidden'}
                        onChange={switchColorBarVisibility}
                    >
                        <FormControlLabel value='hidden' control={<Radio />} label='Hidden' />
                        <FormControlLabel value='visible' control={<Radio />} label='Visible' />
                    </RadioGroup>
                </FormControl>
                <FormControl fullWidth>
                    <FormLabel>Draw order</FormLabel>
                    <RadioGroup
                        row
                        value={drawOrder}
                        onChange={switchDrawOrder}
                    >
                        <FormControlLabel
                            control={<Radio />}
                            value={'selected_unordered'}
                            label={'None'}
                        />
                        <FormControlLabel
                            control={<Radio />}
                            value={'selected_increasing'}
                            label={'Incr.'}
                        />
                        <FormControlLabel
                            control={<Radio />}
                            value={'selected_decreasing'}
                            label={'Decr.'}
                        />
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
                            value={'constant'}
                            label={'Const.'}
                        />
                        <FormControlLabel
                            control={<Radio />}
                            value={'attribute'}
                            label={'Attr.'}
                        />
                        {userGroup === 'PPC' ? <FormControlLabel
                            control={<Radio />}
                            value={'probability'}
                            label={'Prob.'}
                        /> : null}
                    </RadioGroup>
                </FormControl>
                {
                    colorMode === 'constant' ?
                        <FormControl fullWidth disabled={colorMode !== 'constant'}>
                            <FormLabel>Color Mode: Constant</FormLabel>
                            <Slider
                                min={0}
                                max={1.0}
                                step={EPSILON}
                                value={constantColorModeValue}
                                onChange={setConstantColorValue}
                                onChangeCommitted={setConstantColorValue}
                            />
                        </FormControl> : null
                }
                {
                    colorMode === 'attribute' ?
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
                        {['Magma', 'Inferno', 'Plasma', 'Viridis'].map((v) => <FormControlLabel
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

    const isMac = /(Mac OS|MacPPC|MacIntel|Mac_PowerPC|Macintosh|Mac OS X)/.test(navigator.userAgent);
    const symmetric_cp_key = isMac ? 'Control or Option' : 'Control or Alt';

    const actions = [
        {
            icon: <PanToolIcon />, desc: 'Move attribute axis', sec: 'Left mouse button on axis label',
            modesCheck: (m: InteractionMode) => m != InteractionMode.Disabled
        },
        {
            icon: <AddIcon />, desc: 'Create selection', sec: 'Left mouse button on empty axis line',
            modesCheck: (m: InteractionMode) => m == InteractionMode.Compatibility || m == InteractionMode.Full
        },
        {
            icon: <DragHandleIcon />, desc: 'Move selection', sec: 'Left mouse button on selection',
            modesCheck: (m: InteractionMode) => m == InteractionMode.Compatibility || m == InteractionMode.Full
        },
        {
            icon: <DeleteIcon />, desc: 'Delete selection', sec: 'Left click on selection',
            modesCheck: (m: InteractionMode) => m == InteractionMode.Compatibility || m == InteractionMode.Full
        },
        {
            icon: <OpenInFullIcon />, desc: 'Expand/Collapse axis', sec: 'Left click on axis label',
            modesCheck: (m: InteractionMode) => m == InteractionMode.Restricted || m == InteractionMode.Full
        },
        {
            icon: <OpenWithIcon />, desc: 'Move control point', sec: 'Left mouse button on selection control point',
            modesCheck: (m: InteractionMode) => m == InteractionMode.Full
        },
        {
            icon: <DeleteIcon />, desc: 'Delete control point', sec: 'Left click on selection control point',
            modesCheck: (m: InteractionMode) => m == InteractionMode.Full
        },
        {
            icon: <AddIcon />, desc: 'Add control point', sec: 'Drag control point while holding the Shift key',
            modesCheck: (m: InteractionMode) => m == InteractionMode.Full
        },
        {
            icon: <AddIcon />, desc: 'Add symmetric control point', sec: `Drag first/last control point while holding the ${symmetric_cp_key} key`,
            modesCheck: (m: InteractionMode) => m == InteractionMode.Full
        },
    ];

    return (
        <Accordion sx={{ width: '100%' }}>
            <AccordionSummary
                id='actions-info-header'
                aria-controls='actions-info-content'
                expandIcon={<ExpandMoreIcon />}
            >
                <Typography variant='h5'>Actions</Typography>
            </AccordionSummary>
            <AccordionDetails>
                <List dense>
                    {actions.map((a) => a.modesCheck(interactionMode) ? <ListItem>
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

const DebugInfo = (ppc: Props, demo: DemoState, setProps: (newProps) => void,
    logPPCEvent: (newProps) => void) => {
    const { userId, userGroup, currentTask, tasks } = demo;

    const debugShowAxisBB = ppc.debug ? ppc.debug.showAxisBoundingBox : false;
    const debugShowLabelBB = ppc.debug ? ppc.debug.showLabelBoundingBox : false;
    const debugShowCurvesBB = ppc.debug ? ppc.debug.showCurvesBoundingBox : false;
    const debugShowAxisLineBB = ppc.debug ? ppc.debug.showAxisLineBoundingBox : false;
    const debugShowSelectionsBB = ppc.debug ? ppc.debug.showSelectionsBoundingBox : false;
    const debugShowColorBarBB = ppc.debug ? ppc.debug.showColorBarBoundingBox : false;

    let debugItem = null;
    if (demo.showDebugInfo) {
        debugItem = (
            <Accordion sx={{ width: '100%' }}>
                <AccordionSummary
                    id='debug-info-header'
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
                            const debugClone = ppc.debug
                                ? window.structuredClone(ppc.debug)
                                : {
                                    showAxisBoundingBox: false,
                                    showLabelBoundingBox: false,
                                    showCurvesBoundingBox: false,
                                    showAxisLineBoundingBox: false,
                                    showSelectionsBoundingBox: false,
                                    showColorBarBoundingBox: false,
                                };
                            switch (element.value) {
                                case 'axis':
                                    debugClone.showAxisBoundingBox = !debugClone.showAxisBoundingBox;
                                    break;
                                case 'label':
                                    debugClone.showLabelBoundingBox = !debugClone.showLabelBoundingBox;
                                    break;
                                case 'curves':
                                    debugClone.showCurvesBoundingBox = !debugClone.showCurvesBoundingBox;
                                    break;
                                case 'axis_lines':
                                    debugClone.showAxisLineBoundingBox = !debugClone.showAxisLineBoundingBox;
                                    break;
                                case 'selections':
                                    debugClone.showSelectionsBoundingBox = !debugClone.showSelectionsBoundingBox;
                                    break;
                                case 'colorbar':
                                    debugClone.showColorBarBoundingBox = !debugClone.showColorBarBoundingBox;
                                    break;
                            }
                            ppc.debug = debugClone;
                            logPPCEvent({ debug: debugClone });
                            setProps({ ppcState: ppc });
                        }}
                    >
                        <FormLabel>Bounding Boxes</FormLabel>
                        <FormControlLabel control={<Switch checked={debugShowAxisBB} />} value='axis' label='Axis' />
                        <FormControlLabel control={<Switch checked={debugShowLabelBB} />} value='label' label='Label' />
                        <FormControlLabel control={<Switch checked={debugShowCurvesBB} />} value='curves' label='Curves' />
                        <FormControlLabel control={<Switch checked={debugShowAxisLineBB} />} value='axis_lines' label='Axis lines' />
                        <FormControlLabel control={<Switch checked={debugShowSelectionsBB} />} value='selections' label='Selections' />
                        <FormControlLabel control={<Switch checked={debugShowColorBarBB} />} value='colorbar' label='Colorbar' />
                    </FormGroup>
                </AccordionDetails>
            </Accordion>
        )
    }

    return debugItem;
}

const constructTasks = (userGroup: UserGroup, taskMode: TaskMode) => {
    const tasks = [];

    if (taskMode === 'Full' || taskMode === 'Tutorial') {
        tasks.push(tutorial1());
        tasks.push(tutorial2());

        if (userGroup === 'PPC') {
            tasks.push(tutorial2A());
            tasks.push(tutorial2B());
        }

        tasks.push(tutorial3(userGroup))
        tasks.push(tutorial4(userGroup))
        tasks.push(tutorial5(userGroup))

        tasks.push(tutorialFreeRoam(userGroup));
    }

    if (taskMode === 'Full' || taskMode === 'Eval') {
        tasks.push(taskSynthetic(userGroup));
        tasks.push(taskAdult(userGroup));
        tasks.push(taskAblation(userGroup));
    }

    return tasks;
}

const tutorial1 = (): DemoTask => {
    const buildInstructions = [() => {
        return (
            <Stack spacing={1}>
                <DialogContentText>
                    In a parallel coordinates plot, a data point is represented as a curve, passing through
                    all attribute axes. For example, in a three-dimensional dataset with the attributes &#32;
                    <b>A1</b>, <b>A2</b> and <b>A3</b>, the point (5, 10, 3) would be represented as a
                    curve, passing through the value 5 on the axis <b>A1</b>, 10 on the axis <b>A2</b>,
                    and 3 on the axis <b>A3</b>. This enables the visualization of datasets with many
                    attributes.
                    <br />
                    <br />
                    One additional property of a parallel coordinates plot is, that one can estimate
                    correlations between attributes. For example, one could observe that while the value
                    of an attribute increases, it decreases for another attribute. However, this type of
                    analysis is only possible, when the two attributes that we want to compare are direct
                    neighbors. Therefore, the order of the attribute axes is significant, and it is
                    possible to reorder them at will.
                </DialogContentText>
                <video autoPlay loop muted height={420} style={{ objectFit: 'fill' }} id='instructions_video'>
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
    }];

    return {
        name: 'Reorder attribute axes.',
        shortDescription: 'Reorder A1 in between A2 and A3',
        instructions: buildInstructions,
        viewed: true,
        initialState: {
            axes: {
                'a1': {
                    label: 'A1',
                    range: [0, 10000],
                    dataPoints: [...Array(100)].map((v, x) => x * x),
                },
                'a2': {
                    label: 'A2',
                    range: [0, 10],
                    dataPoints: [...Array(100)].map(() => Math.random() * 10),
                },
                'a3': {
                    label: 'A3',
                    range: [0, 100],
                    dataPoints: [...Array(100)].map((v, x) => 100 - x),
                },
            },
            order: ['a1', 'a2', 'a3'],
            labels: {
                'Default': {},
            },
            activeLabel: 'Default',
            colors: {
                selected: {
                    scale: 'plasma',
                    color: 0.5,
                }
            },
            colorBar: 'hidden',
            interactionMode: InteractionMode.RestrictedCompatibility,
            powerProfile: 'high',
            setProps: undefined,
        },
        finalState: null,
        canContinue: (ppc: Props) => (ppc.order[0] == 'a2'
            && ppc.order[1] == 'a1' && ppc.order[2] == 'a3')
            || (ppc.order[0] == 'a3' && ppc.order[1] == 'a1'
                && ppc.order[2] == 'a2'),
        disableLabels: true,
        disableAttributes: true,
        disableColors: true,
    };
}

const tutorial2 = (): DemoTask => {
    const buildInstructions = [() => {
        return (
            <Stack spacing={1}>
                <DialogContentText>
                    Another basic interaction in a parallel coordinates plot is the brushing of interesting
                    data points. By brushing on an attribute axis it is possible to filter the data to show
                    which curves pass through the brushed range of values. The curves that are filtered out
                    will be shown in a light gray color. Multiple brushes on the same axis will filter
                    curves that pass through at least one of them.
                </DialogContentText>
                <video autoPlay loop muted height={420} style={{ objectFit: 'fill' }} id='instructions_video'>
                    <source src={brushingInstr} type='video/mp4'></source>
                </video>
                <DialogContentText>
                    New selections can be brushed by holding the left mouse button over an unbrushed portion
                    of an attribute axis and dragging the mouse. Once added, the brush can be removed by
                    clicking the left mouse button, while hovering the brush. Otherwise, it is possible to
                    move the brush by holding the left mouse button and dragging the pointer.
                    <br />
                    <br />
                    Select the ranges <b>10 to 20</b> and <b>80 to 90</b> of the attribute <b>A2</b>, without
                    including any curve passing though the range <b>40 to 60</b> of the same attribute.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right once the task has been completed.
                </DialogContentText>
            </Stack>);
    }];

    const checkSelectedRanges = (selections?: { [id: string]: Brushes }) => {
        if (!selections || 'Default' in selections === false) {
            return false;
        }

        const brushes = selections['Default'];
        if ('a1' in brushes || 'a3' in brushes || 'a2' in brushes === false) {
            return false;
        }
        const a2 = brushes['a2'];

        const mustContain: [number, boolean][] = [
            [10, false],
            [15, false],
            [20, false],
            [80, false],
            [85, false],
            [90, false],
        ];
        const mustNotContain: [number, boolean][] = [
            [40, false],
            [45, false],
            [50, false],
            [55, false],
            [60, false],
        ];
        for (const brush of a2) {
            const min = brush.controlPoints[0][0];
            const max = brush.controlPoints[brush.controlPoints.length - 1][0];
            for (const x of mustContain) {
                if (min <= x[0] && x[0] <= max) {
                    x[1] = true;
                }
            }
            for (const x of mustNotContain) {
                if (min <= x[0] && x[0] <= max) {
                    x[1] = true;
                }
            }
        }

        const allMust = mustContain
            .map(([v, contained]) => contained)
            .reduce((curr, x) => curr && x, true);
        const anyMustNot = mustNotContain
            .map(([v, contained]) => contained)
            .reduce((curr, x) => curr || x, false);

        return allMust && !anyMustNot;
    }

    return {
        name: 'Create a new selection.',
        shortDescription: 'Select only the ranges [10, 20] and [80, 90] of A2.',
        instructions: buildInstructions,
        viewed: false,
        initialState: {
            axes: {
                'a1': {
                    label: 'A1',
                    range: [0, 10000],
                    dataPoints: [...Array(100)].map((v, x) => x * x),
                },
                'a2': {
                    label: 'A2',
                    range: [0, 100],
                    dataPoints: [...Array(100)].map((v, x) => 100 - x),
                },
                'a3': {
                    label: 'A3',
                    range: [-125000, 125000],
                    dataPoints: [...Array(100)].map((v, x) => Math.pow(x - 50, 3)),
                },
            },
            order: ['a1', 'a2', 'a3'],
            labels: {
                'Default': {},
            },
            activeLabel: 'Default',
            colors: {
                selected: {
                    scale: 'plasma',
                    color: 0.5,
                }
            },
            colorBar: 'hidden',
            interactionMode: InteractionMode.Compatibility,
            powerProfile: 'high',
            setProps: undefined,
        },
        finalState: null,
        canContinue: (ppc: Props) => checkSelectedRanges(ppc.brushes),
        disableLabels: true,
        disableAttributes: true,
        disableColors: true,
    };
}

const tutorial2A = (): DemoTask => {
    const buildInstructions = [() => {
        return (
            <Stack spacing={1}>
                <DialogContentText>
                    You can expand the axis by clicking the left mouse button, while hovering the label
                    of an attribute. In the expanded state, you can see and modify the degee of certainty
                    with which a data point is part of the selection. The certainty is then shown by a
                    curve, where the values range from <b>0%</b> certainty on the <b>rightmost</b> to
                    <b>100%</b> certainty on the leftmost of the curve. When collapsed, the certainty
                    of a brush is shown through the color of the brush, with <b>neon green</b> indicating
                    a certainty of <b>100%</b> and <b>black</b> indicating a certainty of <b>0%</b>.
                </DialogContentText>
                <video autoPlay loop muted height={420} style={{ objectFit: 'fill' }} id='instructions_video'>
                    <source src={extendAxisInstr} type='video/mp4'></source>
                </video>
            </Stack>);
    },
    () => {
        return (
            <Stack spacing={1}>
                <DialogContentText>
                    Each new brush is added with a certainty of <b>100%</b>. The certainty can then be
                    modified, by moving the <b>control points</b> on the curve with the <b>left mouse
                        button</b>. When multiple brushes overlap, the curve will be formed by computing
                    the maxium of all overlapping segments. The individual brushes are not combined
                    in the expanded mode.
                </DialogContentText>
                <video autoPlay loop muted height={420} style={{ objectFit: 'fill' }} id='instructions_video'>
                    <source src={brushingExtInstr} type='video/mp4'></source>
                </video>
            </Stack>);
    },
    () => {
        return (
            <Stack spacing={1}>
                <DialogContentText>
                    You can insert a new control point to a brush, by holding the <b>Shift</b> key, before
                    dragging a control point on the axis. Alternatively, you can add the control point on
                    both ends of the brushed region by holding either of the <b>Ctrl</b>, <b>Alt</b>,
                    or <b>Option</b> keys, before dragging the first or last control points of the brush.
                    A control point can be removed by clicking it on the axis.
                </DialogContentText>
                <video autoPlay loop muted height={420} style={{ objectFit: 'fill' }} id='instructions_video'>
                    <source src={brushFadeoutInstr} type='video/mp4'></source>
                </video>
            </Stack>);
    },
    () => {
        return (
            <Stack spacing={1}>
                <DialogContentText>
                    Create one brush on the attribute <b>A2</b>, consisting of 4 control points.
                    The brush must start at around the value <b>30</b> and must include all curves
                    up to the value <b>70</b>. The certainty on the first and last control point
                    must be <b>0%</b>, while the range <b>40</b> to <b>60</b> must have a
                    certainty of <b>100%</b>.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right once the task has been completed.
                </DialogContentText>
            </Stack>);
    }];

    const checkSelectedRanges = (selections?: { [id: string]: Brushes }) => {
        if (!selections || 'Default' in selections === false) {
            return false;
        }

        const brushes = selections['Default'];
        if ('a1' in brushes || 'a3' in brushes || 'a2' in brushes === false) {
            return false;
        }
        const a2 = brushes['a2'];
        if (a2.length != 1) {
            return false;
        }

        const brush = a2[0];
        if (brush.controlPoints.length != 4) {
            return false;
        }

        const [c1, c2, c3, c4] = brush.controlPoints;
        if (c1[1] != 0 || c2[1] != 1 || c3[1] != 1 || c4[1] != 0) {
            return false;
        }

        if (c1[0] < 25 || c1[0] > 30) {
            return false;
        }
        if (c2[0] < 35 || c2[0] > 45) {
            return false;
        }
        if (c3[0] < 55 || c3[0] > 65) {
            return false;
        }
        if (c4[0] < 70 || c4[0] > 75) {
            return false;
        }

        return true;
    }

    return {
        name: 'Uncertain brushing.',
        shortDescription: 'Brush A2 in the range [30, 70], where the range [40, 60] has a certainty of 100%.',
        instructions: buildInstructions,
        viewed: false,
        initialState: {
            axes: {
                'a1': {
                    label: 'A1',
                    range: [0, 10000],
                    dataPoints: [...Array(100)].map((v, x) => x * x),
                },
                'a2': {
                    label: 'A2',
                    range: [0, 100],
                    dataPoints: [...Array(100)].map((v, x) => 100 - x),
                },
                'a3': {
                    label: 'A3',
                    range: [-125000, 125000],
                    dataPoints: [...Array(100)].map((v, x) => Math.pow(x - 50, 3)),
                },
            },
            order: ['a1', 'a2', 'a3'],
            labels: {
                'Default': {},
            },
            activeLabel: 'Default',
            colors: {
                selected: {
                    scale: 'plasma',
                    color: 0.5,
                }
            },
            colorBar: 'hidden',
            interactionMode: InteractionMode.Full,
            powerProfile: 'high',
            setProps: undefined,
        },
        finalState: null,
        canContinue: (ppc: Props) => checkSelectedRanges(ppc.brushes),
        disableLabels: true,
        disableAttributes: true,
        disableColors: true,
    };
}

const tutorial2B = (): DemoTask => {
    const buildInstructions = [() => {
        return (
            <Stack spacing={1}>
                <DialogContentText>
                    You can control how the curve behaves in between the control points by selecting
                    a different interpolation mode on the top right toolbar. Different interpolation
                    modes may allow you to better approximate the shape of a desired curve. The
                    interpolation modes are (from left to right): <b>Linear</b>, <b>In</b>, <b>Out</b>,
                    and <b>In-Out</b>. By default, the linear interpolation mode is selected. The
                    interpolation mode of the primary segment of each brush is always linear, and can
                    not be changed.
                </DialogContentText>
                <video autoPlay loop muted height={420} style={{ objectFit: 'fill' }} id='instructions_video'>
                    <source src={interpolationChangeInstr} type='video/mp4'></source>
                </video>
                <DialogContentText>
                    Set the interpolation mode to <b>In-Out</b>.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right once the task has been completed.
                </DialogContentText>
            </Stack>);
    }];

    const checkInterpolationMode = (labels: { [id: string]: LabelInfo }) => {
        const label = labels['Default'];
        return label.easing === 'inout';
    };

    return {
        name: 'Different curves.',
        shortDescription: 'Switch the interpolation mode to In-Out.',
        instructions: buildInstructions,
        viewed: false,
        initialState: {
            axes: {
                'a1': {
                    label: 'A1',
                    range: [0, 10000],
                    dataPoints: [...Array(100)].map((v, x) => x * x),
                },
                'a2': {
                    label: 'A2',
                    range: [0, 100],
                    dataPoints: [...Array(100)].map((v, x) => 100 - x),
                },
                'a3': {
                    label: 'A3',
                    range: [-125000, 125000],
                    dataPoints: [...Array(100)].map((v, x) => Math.pow(x - 50, 3)),
                },
            },
            order: ['a1', 'a2', 'a3'],
            labels: {
                'Default': {},
            },
            activeLabel: 'Default',
            colors: {
                selected: {
                    scale: 'plasma',
                    color: 0.5,
                }
            },
            colorBar: 'hidden',
            brushes: {
                'Default': {
                    'a2': [
                        { controlPoints: [[30, 0], [40, 1], [60, 1], [70, 0]], mainSegmentIdx: 1 }
                    ]
                }
            },
            interactionMode: InteractionMode.Full,
            powerProfile: 'high',
            setProps: undefined,
        },
        finalState: null,
        canContinue: (ppc: Props) => checkInterpolationMode(ppc.labels),
        disableLabels: true,
        disableAttributes: true,
        disableColors: true,
    };
}

const tutorial3 = (userGroup: UserGroup): DemoTask => {
    const interactionMode = userGroup === 'PC'
        ? InteractionMode.Compatibility
        : InteractionMode.Full;

    const buildInstructions = [() => {
        return (
            <Stack spacing={1}>
                <DialogContentText>
                    Sometimes it may be required to assign multiple labels to the filtered data points.
                    This can be achieved by creating a new label, and brushing the curves that should
                    be assigned to the new label. The brushes are only valid for the currently active
                    label. The active label can be changed by pressing the play button next to the
                    label name.
                </DialogContentText>
                <video autoPlay loop muted height={420} style={{ objectFit: 'fill' }} id='instructions_video'>
                    <source src={labelsNewInstr} type='video/mp4'></source>
                </video>
            </Stack>);
    }];

    if (userGroup === 'PPC') {
        buildInstructions.push(() => {
            return (
                <Stack spacing={1}>
                    <DialogContentText>
                        Whether a data point counts as selected is decided by the computed certainty and
                        the requested certainty range. By default, the parallel coordinates plot selects
                        any data point, regardless of the certainty. You can configure the certainty
                        required to count as selected, with the slider unter the <b>Labels</b> section.
                        Different labels can have different certainty bounds.
                    </DialogContentText>
                    <video autoPlay loop muted height={420} style={{ objectFit: 'fill' }} id='instructions_video'>
                        <source src={labelsCertaintyInstr} type='video/mp4'></source>
                    </video>
                </Stack>);
        });
        buildInstructions.push(() => {
            return (
                <Stack spacing={1}>
                    <DialogContentText>
                        When the attribute axis is expanded, you can see the curves of the other labels,
                        along with the curve of the currently active label.
                    </DialogContentText>
                    <video autoPlay loop muted height={420} style={{ objectFit: 'fill' }} id='instructions_video'>
                        <source src={labelsCompareInstr} type='video/mp4'></source>
                    </video>
                </Stack>);
        });
    }

    buildInstructions.push(() => {
        return (
            <Stack spacing={1}>
                <DialogContentText>
                    Create a new label called <b>My Label</b>.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right once the task has been completed.
                </DialogContentText>
            </Stack>);
    });

    const checkLabels = (labels: { [id: string]: LabelInfo }) => {
        return 'My Label' in labels;
    };

    return {
        name: 'Multiple Labels.',
        shortDescription: 'Create a new label called. \'My Label\'.',
        instructions: buildInstructions,
        viewed: false,
        initialState: {
            axes: {
                'a1': {
                    label: 'A1',
                    range: [0, 10000],
                    dataPoints: [...Array(100)].map((v, x) => x * x),
                },
                'a2': {
                    label: 'A2',
                    range: [0, 100],
                    dataPoints: [...Array(100)].map((v, x) => 100 - x),
                },
                'a3': {
                    label: 'A3',
                    range: [-125000, 125000],
                    dataPoints: [...Array(100)].map((v, x) => Math.pow(x - 50, 3)),
                },
            },
            order: ['a1', 'a2', 'a3'],
            labels: {
                'Default': {},
            },
            activeLabel: 'Default',
            colors: {
                selected: {
                    scale: 'plasma',
                    color: 0.5,
                }
            },
            colorBar: 'hidden',
            brushes: {
                'Default': {
                    'a2': [
                        { controlPoints: [[40, 1], [60, 1]], mainSegmentIdx: 0 }
                    ],
                    'a3': [
                        { controlPoints: [[-50000, 1], [0, 1]], mainSegmentIdx: 0 }
                    ]
                }
            },
            interactionMode,
            powerProfile: 'high',
            setProps: undefined,
        },
        finalState: null,
        canContinue: (ppc: Props) => checkLabels(ppc.labels),
        disableAttributes: true,
        disableColors: true,
    };
}

const tutorial4 = (userGroup: UserGroup): DemoTask => {
    const interactionMode = userGroup === 'PC'
        ? InteractionMode.Compatibility
        : InteractionMode.Full;

    const buildInstructions = [() => {
        return (
            <Stack spacing={1}>
                <DialogContentText>
                    The parallel coordinates plot can encode some information in the form of the colors
                    of the data point curves. Depending on the task, it may be worthwhile to look into
                    the color settings under the <b>Colors</b> section, on the right. There you can
                    configure the color bar visibility, the information that is encoded as the color,
                    and the color scale that should be used to color the information.
                </DialogContentText>
                <video autoPlay loop muted height={420} style={{ objectFit: 'fill' }} id='instructions_video'>
                    <source src={colorsInstr} type='video/mp4'></source>
                </video>
            </Stack>);
    }];

    if (userGroup === 'PPC') {
        buildInstructions.push(() => {
            return (
                <Stack spacing={1}>
                    <DialogContentText>
                        In addition to the other color modes, you can select to color the curves based on
                        their computed certainty of selection.
                    </DialogContentText>
                    <video autoPlay loop muted height={420} style={{ objectFit: 'fill' }} id='instructions_video'>
                        <source src={colorsCertaintyInstr} type='video/mp4'></source>
                    </video>
                </Stack>);
        });
        buildInstructions.push(() => {
            return (
                <Stack spacing={1}>
                    <DialogContentText>
                        You may notice, that the parallel coordinates may suffer from a cluttering problem,
                        where, due to overlapping curves, it becomes difficult to see the color of some
                        group of curves. To alleviate this, we allow you to specify an ordering for the curves.
                    </DialogContentText>
                    <video autoPlay loop muted height={420} style={{ objectFit: 'fill' }} id='instructions_video'>
                        <source src={colorsOrderInstr} type='video/mp4'></source>
                    </video>
                </Stack>);
        });
    }

    buildInstructions.push(() => {
        return (
            <Stack spacing={1}>
                <DialogContentText>
                    Enable the color bar under the <b>Colors</b> section.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right once the task has been completed.
                </DialogContentText>
            </Stack>);
    });

    return {
        name: 'Color settings.',
        shortDescription: 'Enable the color bar.',
        instructions: buildInstructions,
        viewed: false,
        initialState: {
            axes: {
                'a1': {
                    label: 'A1',
                    range: [0, 10000],
                    dataPoints: [...Array(100)].map((v, x) => x * x),
                },
                'a2': {
                    label: 'A2',
                    range: [0, 100],
                    dataPoints: [...Array(100)].map((v, x) => 100 - x),
                },
                'a3': {
                    label: 'A3',
                    range: [-125000, 125000],
                    dataPoints: [...Array(100)].map((v, x) => Math.pow(x - 50, 3)),
                },
            },
            order: ['a1', 'a2', 'a3'],
            labels: {
                'Default': {},
            },
            activeLabel: 'Default',
            colors: {
                selected: {
                    scale: 'plasma',
                    color: 0.5,
                }
            },
            colorBar: 'hidden',
            interactionMode,
            powerProfile: 'high',
            setProps: undefined,
        },
        finalState: null,
        canContinue: (ppc: Props) => ppc.colorBar === 'visible',
        disableAttributes: true,
    };
}

const tutorial5 = (userGroup: UserGroup): DemoTask => {
    const interactionMode = userGroup === 'PC'
        ? InteractionMode.Compatibility
        : InteractionMode.Full;

    const buildInstructions = [() => {
        return (
            <Stack spacing={1}>
                <DialogContentText>
                    A dataset may contain more attributes than can be visualized simultaneously.
                    In that case, it may be required to filter which attributes can contribute
                    to the selection of the curves. The <b>Attributes</b> section contains info
                    about all the attributes present in the dataset. There you can select whether
                    to show or hide an attribute from the plot.
                </DialogContentText>
                <video autoPlay loop muted height={420} style={{ objectFit: 'fill' }} id='instructions_video'>
                    <source src={attributesInstr} type='video/mp4'></source>
                </video>
                <DialogContentText>
                    Enable the attribute <b>A5</b> under the <b>Attributes</b> section.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right once the task has been completed.
                </DialogContentText>
            </Stack>);
    }];

    const checkVisible = (axesOrder: string[]) => {
        return axesOrder && axesOrder.indexOf('a5') !== -1;
    };

    return {
        name: 'More attributes.',
        shortDescription: 'Enable the attribute A5.',
        instructions: buildInstructions,
        viewed: false,
        initialState: {
            axes: {
                'a1': {
                    label: 'A1',
                    range: [0, 10000],
                    dataPoints: [...Array(100)].map((v, x) => x * x),
                },
                'a2': {
                    label: 'A2',
                    range: [0, 100],
                    dataPoints: [...Array(100)].map((v, x) => 100 - x),
                },
                'a3': {
                    label: 'A3',
                    range: [-125000, 125000],
                    dataPoints: [...Array(100)].map((v, x) => Math.pow(x - 50, 3)),
                },
                'a4': {
                    label: 'A4',
                    range: [0, 50],
                    dataPoints: [...Array(100)].map(() => Math.random() * 50),
                },
                'a5': {
                    label: 'A5',
                    range: [-10000, 0],
                    dataPoints: [...Array(100)].map((v, x) => -Math.pow(x, 2)),
                },
                'a6': {
                    label: 'A6',
                    range: [0, 10],
                    dataPoints: [...Array(100)].map((v, x) => x / 10),
                },
            },
            order: ['a1', 'a2', 'a3'],
            labels: {
                'Default': {},
            },
            activeLabel: 'Default',
            colors: {
                selected: {
                    scale: 'plasma',
                    color: 0.5,
                }
            },
            colorBar: 'hidden',
            interactionMode,
            powerProfile: 'high',
            setProps: undefined,
        },
        finalState: null,
        canContinue: (ppc: Props) => checkVisible(ppc.order),
    };
}

const tutorialFreeRoam = (userGroup: UserGroup): DemoTask => {
    const interactionMode = userGroup === 'PC'
        ? InteractionMode.Compatibility
        : InteractionMode.Full;

    const buildInstructions = [() => {
        return (
            <>
                <DialogContentText>
                    You have reached the end of the tutorial section. Before you continue to the next task,
                    you can try out interacting with the visualization freely.
                    Remember to look at the <b>Actions</b> tab to see the actions available to you.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right once you feel ready to continue with
                    the next task.
                </DialogContentText>
            </>);
    }];

    return {
        name: 'Make yourself familiar with the visualization.',
        shortDescription: 'Continue when you feel ready.',
        instructions: buildInstructions,
        viewed: false,
        initialState: {
            axes: {
                'a1': {
                    label: 'A1',
                    range: [0, 10000],
                    dataPoints: [...Array(100)].map((v, x) => x * x),
                },
                'a2': {
                    label: 'A2',
                    range: [0, 100],
                    dataPoints: [...Array(100)].map((v, x) => 100 - x),
                },
                'a3': {
                    label: 'A3',
                    range: [-125000, 125000],
                    dataPoints: [...Array(100)].map((v, x) => Math.pow(x - 50, 3)),
                },
                'a4': {
                    label: 'A4',
                    range: [0, 50],
                    dataPoints: [...Array(100)].map(() => Math.random() * 50),
                },
                'a5': {
                    label: 'A5',
                    range: [-10000, 0],
                    dataPoints: [...Array(100)].map((v, x) => -Math.pow(x, 2)),
                },
                'a6': {
                    label: 'A6',
                    range: [0, 10],
                    dataPoints: [...Array(100)].map((v, x) => x / 10),
                },
            },
            order: ['a3', 'a2', 'a1'],
            labels: {
                'Default': {},
            },
            activeLabel: 'Default',
            colors: {
                selected: {
                    scale: 'plasma',
                    color: 0.5,
                }
            },
            colorBar: 'hidden',
            interactionMode: interactionMode,
            debug: {
                showAxisBoundingBox: false,
                showLabelBoundingBox: false,
                showCurvesBoundingBox: false,
                showAxisLineBoundingBox: false,
                showSelectionsBoundingBox: false,
                showColorBarBoundingBox: false,
            },
            powerProfile: 'high',
            setProps: undefined,
        },
        finalState: null,
        canContinue: (ppc: Props) => true
    };
}

const taskSynthetic = (userGroup: UserGroup): DemoTask => {
    const interactionMode = userGroup === 'PC'
        ? InteractionMode.Compatibility
        : InteractionMode.Full;

    const buildInstructions = [() => {
        return (
            <>
                <DialogContentText>
                    For this task, you will look into a synthetic dataset consisting
                    of the attributes <i>A1</i>, <i>A2</i> and <i>Class</i>, where
                    the last attribute denotes the assigned class of each point.
                    The class of each point is determined by assigning a probability
                    to each value of the two attributes and treating them as
                    statistically independent random variables. A point is then
                    assigned to class <b>C1</b>, if the combination of the two
                    random variables yields a probability greater or equal to <b>50%</b>.
                    Otherwise, the point is assigned to the class <b>C2</b>.
                </DialogContentText>
            </>);
    },
    () => {
        return (
            <>
                <DialogContentText>
                    Given the provided information, select the entries assigned to
                    class <b>C1</b>. You may not apply any brush directly to the
                    included <i>Class</i> attribute, but you may use it otherwise.
                    You may estimate the distribution of an attribute by changing the color
                    mode to encode the value of said attribute. For the selection, you must
                    try to maximize the number of entries that truly belong to
                    class <b>C1</b>, while minimizing the number of entries wrongly
                    attributed to that class.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right, once you feel
                    that you have fulfilled the task.
                </DialogContentText>
            </>);
    }];

    const visible = ['a1', 'a2', 'class'];
    const included = [];
    const initialState = syntheticDataset(visible, included);
    initialState.interactionMode = interactionMode;
    initialState.labels = { 'Default': {} };
    initialState.activeLabel = 'Default';
    initialState.colors = {
        selected: { scale: 'plasma', color: 0.5 }
    };
    initialState.colorBar = 'visible';
    initialState.powerProfile = 'high';

    const checkCompleted = (brushes?: { [id: string]: Brushes }) => {
        if (!brushes) {
            return false;
        }

        let hasBrushed = false;
        for (const [_, labelBrushes] of Object.entries(brushes)) {
            if ('class' in labelBrushes) {
                return false;
            }
            hasBrushed = hasBrushed || Object.keys(labelBrushes).length != 0;
        }

        return hasBrushed;
    }

    return {
        name: 'Statistically independent variables.',
        shortDescription: 'Select the entries assigned to class C1.',
        instructions: buildInstructions,
        viewed: false,
        initialState,
        finalState: null,
        canContinue: (ppc: Props) => checkCompleted(ppc.brushes)
    };
}

const taskAdult = (userGroup: UserGroup): DemoTask => {
    const interactionMode = userGroup === 'PC'
        ? InteractionMode.Compatibility
        : InteractionMode.Full;

    const buildInstructions = [() => {
        return (
            <>
                <DialogContentText>
                    For the next task, we will look at a subset of the US
                    Adult Census dataset, which is derived from the 1994
                    US Census database.
                    <br />
                    <br />
                    The dataset tracks various attributes of multiple people,
                    including their age, education, race, marital status,
                    occupation, et cetera, with the aim of predicting whether
                    someone has an annual income greater than $50,000. For this
                    task, you will provided with the age, sex, education and
                    hours worked per week of 5000 random people contained in
                    the dataset.
                </DialogContentText>
            </>);
    },
    () => {
        return (
            <>
                <DialogContentText>
                    Given the provided information, select the entries with an
                    income greater than $50,000. You may not apply any brush directly
                    to the included <i>Income</i> attribute, but you may use it otherwise.
                    You may estimate the distribution of an attribute by changing the color
                    mode to encode the value of said attribute. For the selection, you must
                    try to maximize the number of people who truly have an income greater
                    than $50,000, while minimizing the number of people who are wrongly
                    attributed that label.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right, once you feel
                    that you have fulfilled the task.
                </DialogContentText>
            </>);
    }];

    const visible = ['age', 'sex', 'education', 'hours-per-week', 'income'];
    const included = [];
    const initialState = adultDataset(visible, included, 5000);
    initialState.interactionMode = interactionMode;
    initialState.labels = { 'Default': {} };
    initialState.activeLabel = 'Default';
    initialState.colors = {
        selected: { scale: 'plasma', color: 0.5 }
    };
    initialState.colorBar = 'visible';
    initialState.powerProfile = 'high';

    const checkCompleted = (brushes?: { [id: string]: Brushes }) => {
        if (!brushes) {
            return false;
        }

        let hasBrushed = false;
        for (const [_, labelBrushes] of Object.entries(brushes)) {
            if ('income' in labelBrushes) {
                return false;
            }
            hasBrushed = hasBrushed || Object.keys(labelBrushes).length != 0;
        }

        return hasBrushed;
    }

    return {
        name: 'Filter by income.',
        shortDescription: 'Select the persons with an income greater than 50K.',
        instructions: buildInstructions,
        viewed: false,
        initialState,
        finalState: null,
        canContinue: (ppc: Props) => checkCompleted(ppc.brushes)
    };
}

const taskAblation = (userGroup: UserGroup): DemoTask => {
    const interactionMode = userGroup === 'PC'
        ? InteractionMode.Compatibility
        : InteractionMode.Full;

    const buildInstructions = [() => {
        return (
            <>
                <DialogContentText>
                    For the next task, we will look into a simulated radiofrequency
                    ablation dataset. Radiofrequency ablation is a minimally
                    invasive procedure that aims to remove malicious or dysfunctional
                    tissue, like tumors. A needle-like probe is inserted into the
                    malicious tissue and is then powered, which exposes the cells
                    to a temperature above 60°C. Those temperatures result in the
                    death of the affected cells, when applied for a few minutes.
                    In the treatment of tumorous tissue, the procedure aims to
                    ablate all tumorous cells, including a safety margin of 5 to 10mm.
                    Apart from ablating the malicious tissue and the safety margin
                    around it, the procedure should minimize the harm to the
                    healthy tissue. To that extent, it is important to understand
                    how the biological properties of the involved tissues affect
                    the treatment.
                    <br />
                    <br />
                    The dataset simulates multiple radiofrequency ablation treatments
                    of tumorous liver tissue. Along with the liver and tumor tissues,
                    there are also blood vessels that have to be considered. For each
                    of those tissues, we track the <i>Density</i>, <i>Heat Capacity</i>,
                    &#32;<i>Thermal Conductivity</i> and <i>Blood Perfusion Rate</i>.
                    To quantify the effectiveness of the treatment, we computed
                    the <i>Ablation Volume</i> in mm<sup>3</sup>.
                </DialogContentText>
            </>);
    },
    () => {
        return (
            <>
                <DialogContentText>
                    Given the provided information, select the biological properties that
                    tend to increase the <i>Ablation Volume</i>. You may not apply any brush directly
                    to the included <i>Ablation Volume</i> attribute, but you may use it otherwise.
                    You may estimate the distribution of an attribute by changing the color
                    mode to encode the value of said attribute.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right, once you feel
                    that you have fulfilled the task.
                </DialogContentText>
            </>);
    }];

    const visible = [
        'density_liver',
        'density_vessel',
        'density_tumor',
        'heat_capacity_liver',
        'heat_capacity_vessel',
        'heat_capacity_tumor',
        'thermal_conductivity_liver',
        'thermal_conductivity_vessel',
        'thermal_conductivity_tumor',
        'relative_blood_perfusion_rate_liver',
        'relative_blood_perfusion_rate_vessel',
        'relative_blood_perfusion_rate_tumor',
        'ablation_volume',
    ];
    const included = [];
    const initialState = ablationDataset(visible, included, 5000);
    initialState.interactionMode = interactionMode;
    initialState.labels = { 'Default': {} };
    initialState.activeLabel = 'Default';
    initialState.colors = {
        selected: { scale: 'plasma', color: 0.5 }
    };
    initialState.colorBar = 'visible';
    initialState.powerProfile = 'high';

    const checkCompleted = (brushes?: { [id: string]: Brushes }) => {
        if (!brushes) {
            return false;
        }

        let hasBrushed = false;
        for (const [_, labelBrushes] of Object.entries(brushes)) {
            if ('dice_coefficient' in labelBrushes) {
                return false;
            }
            hasBrushed = hasBrushed || Object.keys(labelBrushes).length != 0;
        }

        return hasBrushed;
    }

    return {
        name: 'Ablation analysis.',
        shortDescription: 'Make a selection that tends to increase the Ablation Volume.',
        instructions: buildInstructions,
        viewed: false,
        initialState,
        finalState: null,
        canContinue: (ppc: Props) => checkCompleted(ppc.brushes)
    };
}