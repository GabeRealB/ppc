/* eslint no-magic-numbers: 0 */
import { S3Client, PutObjectCommand } from '@aws-sdk/client-s3';
import React, { Component, createElement, useEffect, useState } from 'react';
import { v4 as uuid } from 'uuid';
import pako from 'pako';

import { TextareaAutosize as BaseTextareaAutosize } from '@mui/base/TextareaAutosize';
import { styled } from '@mui/system';

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
import Skeleton from '@mui/material/Skeleton';
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

import PPC from '../components/PPC';
import { Axis, Props, InteractionMode, Brushes, LabelInfo } from '../types'

import { syntheticTestDataset, applicationDataset, validationDataset, derivationDataset } from './datasets';

import taskApplicationProbabilityCurves from './resources/task_application_probability_curves.png'
import taskEvaluationCurvesA from './resources/task_evaluation_curves_a.png'
import taskEvaluationCurvesB from './resources/task_evaluation_curves_b.png'

const EPSILON = 1.17549435082228750797e-38;
const VERSION = 2;

type StudyTask = {
    name: string,
    shortDescription: string,
    instructions: (() => React.JSX.Element)[],
    taskResultInput?: (props: { task: StudyTask, forceUpdate: () => void }) => React.JSX.Element,
    taskResult?: any,
    viewed: boolean,
    initialState: Props,
    finalState: Props,
    canContinue: (ppc: Props, task?: StudyTask) => boolean,
    disableLabels?: boolean,
    disableAttributes?: boolean,
    disableColors?: boolean,
};

type UserGroup = 'PC' | 'PPC';

type TaskMode = 'Full' | 'Tutorial' | 'Eval' | 'Paper'

type DemoPage = 'welcome'
    | 'qualitative'
    | 'quantitative'
    | 'feedback'
    | 'finish';

type Sex = 'male' | 'female' | 'other';

type LevelOfEducation = 'tertiary'
    | 'bachelor'
    | 'master'
    | 'doctoral'

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

type Proficiency = 'na'
    | 'fundamental'
    | 'novice'
    | 'intermediate'
    | 'advanced'
    | 'expert'

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
    major?: string,
    colorAbnormality?: ColorAbnormality,
    analysisProficiency?: Proficiency,
    pcProficiency?: Proficiency,
    feedback?: string,
    taskLogs: TaskLog[],
};

type StudyState = {
    currentPage: DemoPage,

    userId: uuid,
    userGroup: UserGroup,
    variant: string,

    currentTask: number,
    tasks: StudyTask[],
    showInstructions: boolean,

    showDebugInfo: boolean,
    dryRun: boolean,

    deadline: Date,
    deadlinePassed: boolean,

    results: Results,
};

type AppState = {
    ppcState: Props,
    demo: StudyState,
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
        if (!['Full', 'Tutorial', 'Eval', 'Paper'].includes(taskMode)) {
            taskMode = 'Full' as TaskMode;
        }
        let userGroup = searchParams.get('userGroup');
        if (userGroup !== 'PC' && userGroup !== 'PPC') {
            userGroup = Math.random() < 0.5 ? 'PC' : 'PPC';
        }
        let variant = searchParams.get('v');
        switch (variant) {
            case 'oudn':
                variant = 'local';
                break;
            case 'dldz':
                variant = 'experts';
                break;
            case 'gtfj':
                variant = 'paper';
                break;
            default:
                variant = 'unknown';
                break;
        }

        const deadline = new Date(2024, 8, 31);
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
                variant,
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
            case 'qualitative':
                page = createElement(QualitativeStudyPage, this);
                break;
            case 'quantitative':
                page = createElement(QuantitativeStudy, this);
                break;
            case 'feedback':
                page = createElement(FeedbackPage, this);
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
            try {
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
            } catch (error) {
                setWebgpuTestStatus(false);
            }
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
        demo.currentPage = 'qualitative';
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

function QualitativeStudyPage(app: App) {
    const { results } = app.state.demo;

    const [age, setAge] = useState<number>(undefined);
    const [sex, setSex] = useState<Sex>(undefined);
    const [education, setEducation] = useState<LevelOfEducation>(undefined);
    const [major, setMajor] = useState<string>(undefined);
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

    const handleMajorChange = (e) => {
        results.major = e.target.value;
        setMajor(e.target.value);
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
                results.analysisProficiency = 'na';
                break;
            case 2:
                results.analysisProficiency = 'fundamental';
                break;
            case 3:
                results.analysisProficiency = 'novice';
                break;
            case 4:
                results.analysisProficiency = 'intermediate';
                break;
            case 5:
                results.analysisProficiency = 'advanced';
                break;
            case 6:
                results.analysisProficiency = 'expert';
                break;
        }
    }

    const handlePcProficiencyChange = (e, proficiency) => {
        proficiency = proficiency ? proficiency : 0;
        setPcProficiency(proficiency);
        switch (proficiency) {
            case 1:
                results.pcProficiency = 'na';
                break;
            case 2:
                results.pcProficiency = 'fundamental';
                break;
            case 3:
                results.pcProficiency = 'novice';
                break;
            case 4:
                results.pcProficiency = 'intermediate';
                break;
            case 5:
                results.pcProficiency = 'advanced';
                break;
            case 6:
                results.pcProficiency = 'expert';
                break;
        }
    }

    const handleClick = () => {
        const { demo } = app.state;
        demo.currentPage = 'quantitative';
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
        && major !== undefined
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
                        <FormControlLabel value='other' control={<Radio />} label='Other' />
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
                    <InputLabel id='education-label'>Achieved level of education</InputLabel>
                    <Select
                        labelId='education-label'
                        id='education-select'
                        value={education}
                        label='Education'
                        sx={{ m: 1, minWidth: 240 }}
                        onChange={handleEducationChange}
                    >
                        <MenuItem value='tertiary'>High school or lower</MenuItem>
                        <MenuItem value='bachelor'>Bachelor or equivalent</MenuItem>
                        <MenuItem value='master'>Master or equivalent</MenuItem>
                        <MenuItem value='doctoral'>Doctoral or equivalent</MenuItem>
                    </Select>
                </FormControl>
            </Box>
            <Box marginY={1.5}>
                <FormControl>
                    <InputLabel id='education-label'>Degree major</InputLabel>
                    <Select
                        labelId='major-label'
                        id='major-select'
                        value={major}
                        label='Major'
                        sx={{ m: 1, minWidth: 240 }}
                        onChange={handleMajorChange}
                    >
                        <MenuItem value='arts-and-humanities'>Arts and Humanities</MenuItem>
                        <MenuItem value='business'>Business</MenuItem>
                        <MenuItem value='health-and-medicine'>Health and Medicine</MenuItem>
                        <MenuItem value='multi-interdisciplinary-studies'>Multi-Interdisciplinary Studies</MenuItem>
                        <MenuItem value='public-and-social-services'>Public and Social Services</MenuItem>
                        <MenuItem value='stem'>Science, Technology, Engineering, and Math</MenuItem>
                        <MenuItem value='social-sciences'>Social Sciences</MenuItem>
                        <MenuItem value='trades-and-personal-services'>Trades and Personal Services</MenuItem>
                        <MenuItem value='other'>Other</MenuItem>
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

function QuantitativeStudy(app: App) {
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
                <Grid xs={2} maxHeight={'95%'} sx={{ overflowY: 'auto', overflowX: 'hidden' }}>
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

function FeedbackPage(app: App) {
    const { demo } = app.state;
    const { results } = demo;

    const blue = {
        100: '#DAECFF',
        200: '#b6daff',
        400: '#3399FF',
        500: '#007FFF',
        600: '#0072E5',
        900: '#003A75',
    };

    const grey = {
        50: '#F3F6F9',
        100: '#E5EAF2',
        200: '#DAE2ED',
        300: '#C7D0DD',
        400: '#B0B8C4',
        500: '#9DA8B7',
        600: '#6B7A90',
        700: '#434D5B',
        800: '#303740',
        900: '#1C2025',
    };

    const Textarea = styled(BaseTextareaAutosize)(
        ({ theme }) => `
        box-sizing: border-box;
        width: 100%;
        font-family: 'IBM Plex Sans', sans-serif;
        font-size: 0.875rem;
        font-weight: 400;
        line-height: 1.5;
        padding: 8px 12px;
        border-radius: 8px;
        color: ${theme.palette.mode === 'dark' ? grey[300] : grey[900]};
        background: ${theme.palette.mode === 'dark' ? grey[900] : '#fff'};
        border: 1px solid ${theme.palette.mode === 'dark' ? grey[700] : grey[200]};
        box-shadow: 0px 2px 2px ${theme.palette.mode === 'dark' ? grey[900] : grey[50]};

        &:hover {
        border-color: ${blue[400]};
        }

        &:focus {
        border-color: ${blue[400]};
        box-shadow: 0 0 0 3px ${theme.palette.mode === 'dark' ? blue[600] : blue[200]};
        }

        // firefox
        &:focus-visible {
        outline: 0;
        }
        `,
    );

    const handleFeedbackChange = (e) => {
        results.feedback = e.target.value;
    };

    const handleClick = () => {
        demo.currentPage = 'finish';
        app.setProps({ demo });
    };

    return (
        <Container>
            <Typography variant='h4'>
                <b>
                    Before you finish, you can provide some feedback about any
                    difficulties encountered, unclear task descriptions, suggestions
                    and the like.
                </b>
            </Typography>
            <Box marginY={2}>
                <Textarea
                    aria-label="feedback"
                    minRows={5}
                    placeholder="Feedback"
                    onChange={handleFeedbackChange}
                />
            </Box>
            <Box marginY={2}>
                <Button
                    variant='contained'
                    onClick={handleClick}
                    fullWidth
                >
                    Finish test
                </Button>
            </Box>
        </Container>
    );
}

function FinishPage(app: App) {
    const { demo } = app.state;
    const { results, userId, userGroup, variant, deadlinePassed, dryRun } = demo;

    const [finished, setFinished] = useState<{ error: any } | boolean>(deadlinePassed);

    useEffect(() => {
        if (finished) {
            return;
        }

        const fileName = `${uuid()}.bin`;
        const fileContents = { userId, userGroup, variant, results, VERSION };
        const fileContentsJSON = JSON.stringify(fileContents);
        const fileContentsCompressed = pako.deflate(fileContentsJSON);

        if (dryRun) {
            console.info('Dry run results', fileName, fileContents, fileContentsCompressed);
            console.info('Raw length', fileContentsJSON.length);
            console.info('Compressed length', fileContentsCompressed.length);
            setFinished(true);
            return;
        }

        try {
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
                setFinished({ error: e.toString() });
            })
        } catch (e) {
            setFinished({ error: e.toString() });
        }
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
    demo: StudyState;
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
    demo: StudyState,
    setProps: (newProps) => void,
    logPPCEvent: (newProps) => void,
    forceUpdate: () => void,
) => {
    const { currentTask, tasks, results } = demo;
    const task = tasks[currentTask];
    const { name, shortDescription } = task;

    const getSelectedIndices = () => {
        const { selectionIndices } = ppc;
        if (!selectionIndices) {
            return {};
        }

        const selections = {};
        for (const [label, indices] of Object.entries(selectionIndices)) {
            selections[label] = Array.from(indices).map((value) => Number(value));
        }
        return selections;
    };

    const handleNext = () => {
        const { taskLogs } = results;
        const timestamp = performance.now();

        const current = tasks[currentTask];
        const currentLog = taskLogs[currentTask];
        currentLog.events.push({
            type: 'end',
            timestamp,
            data: { 'selected': getSelectedIndices() }
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
            demo.currentPage = 'feedback';
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
            timestamp,
            data: { 'selected': getSelectedIndices() }
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
    demo: StudyState,
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
    demo: StudyState,
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
    demo: StudyState,
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
    const attributeDensityColorModeValue = typeof (ppc.colors?.selected?.color) == 'object' && ppc.colors?.selected?.color.type === 'attribute_density'
        ? ppc.colors?.selected?.color.attribute
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
                if (typeof (colors.selected.color) === 'object') {
                    if (colors.selected.color.type === 'probability') {
                        colorMode = 'probability';
                    } else if (colors.selected.color.type === 'attribute_density') {
                        colorMode = 'attribute_density';
                    }
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
            : { selected: { color: 0.5, scale: 'magma' } };
        switch (colorMode) {
            case 'constant':
                colorsClone.selected.color = constantColorModeValue;
                break;
            case 'attribute':
                colorsClone.selected.color = attributeColorModeValue;
                break;
            case 'attribute_density':
                colorsClone.selected.color = {
                    type: 'attribute_density',
                    attribute: attributeDensityColorModeValue
                };
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
            : { selected: { color: 0.5, scale: 'magma' } };
        colorsClone.drawOrder = drawOrder;
        ppc.colors = colorsClone;
        logPPCEvent({ colors: colorsClone });
        setProps({ ppcState: ppc });
    }

    const setConstantColorValue = (e: Event, value: number) => {
        const colorsClone = colors ?
            window.structuredClone(colors)
            : { selected: { color: 0.5, scale: 'magma' } };
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
            : { selected: { color: 0.5, scale: 'magma' } };
        colorsClone.selected.color = value;
        ppc.colors = colorsClone;
        logPPCEvent({ colors: colorsClone });
        setProps({ ppcState: ppc, demo })
    };

    const setAttributeDensityColorValue = (e, value) => {
        const colorsClone = colors ?
            window.structuredClone(colors)
            : { selected: { color: 0.5, scale: 'magma' } };
        colorsClone.selected.color = {
            type: 'attribute_density',
            attribute: value,
        };
        ppc.colors = colorsClone;
        logPPCEvent({ colors: colorsClone });
        setProps({ ppcState: ppc, demo })
    };

    const setColorMap = (e, colorMap) => {
        const colorsClone = colors ?
            window.structuredClone(colors)
            : { selected: { color: 0.5, scale: 'magma' } };
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
                        <FormControlLabel
                            control={<Radio />}
                            value={'attribute_density'}
                            label={'Dens.'}
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
                {
                    colorMode === 'attribute_density' ?
                        <FormControl fullWidth>
                            <FormLabel>Color Mode: Attribute Density</FormLabel>
                            <RadioGroup
                                row
                                value={attributeDensityColorModeValue}
                                onChange={setAttributeDensityColorValue}
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

const DebugInfo = (ppc: Props, demo: StudyState, setProps: (newProps) => void,
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
        tasks.push(taskTutorial(userGroup));
    }

    if (taskMode === 'Full' || taskMode === 'Eval') {
        tasks.push(taskApplication(userGroup));
        tasks.push(taskValidation(userGroup));
        tasks.push(taskDerivation(userGroup));
    }

    return tasks;
}

const taskTutorial = (userGroup: UserGroup): StudyTask => {
    const interactionMode = userGroup === 'PC'
        ? InteractionMode.Compatibility
        : InteractionMode.Full;

    const buildInstructions = [() => {
        return (
            <>
                <DialogContentText>
                    Before you continue to the first task, you can try out interacting with the visualization freely.
                    Remember to look at the <b>Actions</b> tab to see the actions available to you.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right once you feel ready to continue with
                    the first task.
                </DialogContentText>
            </>);
    }];

    const visible = ['a1', 'class', 'label'];
    const included = [];
    const { state: initialState, sampleIndices } = syntheticTestDataset(visible, included);
    initialState.interactionMode = interactionMode;;
    initialState.labels = { 'Default': {} };
    initialState.activeLabel = 'Default';
    initialState.colors = {
        selected: { scale: 'magma', color: 0.5 }
    };
    initialState.colorBar = 'visible';
    initialState.powerProfile = 'high';

    return {
        name: 'Make yourself familiar with the visualization.',
        shortDescription: 'Continue when you feel ready.',
        instructions: buildInstructions,
        viewed: false,
        initialState,
        finalState: null,
        canContinue: (ppc: Props) => true
    };
}

const taskApplication = (userGroup: UserGroup): StudyTask => {
    const interactionMode = userGroup === 'PC'
        ? InteractionMode.Compatibility
        : InteractionMode.Full;

    const buildInstructions = [() => {
        return (
            <>
                <DialogContentText>
                    For this task, you will look into a synthetic dataset consisting
                    of the attributes <i>A1</i> and <i>A2</i>. This task tests the
                    ability of a user, to faithfully recreate known selections.
                    To that end, you are provided with the probability curves for
                    both attributes <i>A1</i> and <i>A2</i>:
                    <img src={taskApplicationProbabilityCurves} style={{ objectFit: 'fill' }} />
                </DialogContentText>
            </>);
    },
    () => {
        return (
            <>
                <DialogContentText>
                    <b>Task:</b><br />
                    Given the provided information, <b>select the entries that have a
                        selection probability <i>p</i>, with 0% &lt; <i>p</i> &le; 25%</b>.
                    As a reminder - the selection probability of each entry is derived by
                    multiplying the probability of the entry on each individual attribute.
                    For instance, if an entry passes through a point with a selection
                    probability of <b>70%</b> on <i>A1</i> and <b>50%</b> on <i>A2</i>,
                    the final selection probability amounts to <b>35%</b>.
                    <br />
                    Rate your confidence of being able to correctly select the requested
                    entries.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right, once you feel
                    that you have fulfilled the task.
                </DialogContentText>
            </>);
    }];

    const visible = ['a1', 'a2'];
    const included = [];
    const { state: initialState, sampleIndices } = applicationDataset(visible, included, 500);
    initialState.interactionMode = interactionMode;
    initialState.labels = { 'Default': {} };
    initialState.activeLabel = 'Default';
    initialState.colors = {
        selected: { scale: 'magma', color: 0.5 }
    };
    initialState.colorBar = 'visible';
    initialState.powerProfile = 'high';

    const taskResult = {
        sampleIndices,
        confidence: undefined,
    };

    const taskResultInput = (props: { task: StudyTask, forceUpdate: () => void }): React.JSX.Element => {
        const { task, forceUpdate } = props;
        const { taskResult } = task;
        const { confidence } = taskResult;

        const updateOverallConfidence = (e, value) => {
            taskResult.confidence = value ? value : undefined;
            forceUpdate();
        };

        return (
            <>
                <Typography variant='subtitle1' marginY={2}>
                    Rate your confidence:
                </Typography>
                <Container>
                    <Rating
                        name='confidence'
                        value={confidence}
                        max={6}
                        size='large'
                        onChange={updateOverallConfidence}
                        emptyIcon={<StarIcon style={{ opacity: 0.55 }} fontSize='inherit' />}
                    />
                </Container>
            </>);
    };

    const checkCompleted = (brushes?: { [id: string]: Brushes }) => {
        if (!brushes) {
            return false;
        }

        if (taskResult.confidence === undefined) {
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
        name: 'Applying existing probability curves.',
        shortDescription: 'Select the entries with a selection probability p, where 0% < p ≤ 25%.',
        instructions: buildInstructions,
        viewed: false,
        initialState,
        finalState: null,
        taskResult,
        taskResultInput,
        canContinue: (ppc: Props) => checkCompleted(ppc.brushes)
    };
}

const taskValidation = (userGroup: UserGroup): StudyTask => {
    const interactionMode = userGroup === 'PC'
        ? InteractionMode.Compatibility
        : InteractionMode.Full;

    const buildInstructions = [() => {
        return (
            <>
                <DialogContentText>
                    For this task, you will look into another synthetic dataset
                    consisting of the attributes <i>A1</i>, <i>A2</i> and <i>Class</i>.
                    This task tests the ability of a user, identify the set of curves
                    matching the computed classes of the entries. To that end, you are
                    provided with two sets of probability curves (denoted <b>(a)</b>
                    &#32;and <b>(b)</b>):
                    <img src={taskEvaluationCurvesA} style={{ objectFit: 'fill' }} />
                    <img src={taskEvaluationCurvesB} style={{ objectFit: 'fill' }} />
                </DialogContentText>
            </>);
    },
    () => {
        return (
            <>
                <DialogContentText>
                    <b>Task:</b><br />
                    Given the two sets of probability curves, <b>select whether the
                        curves (a) or (b) better fit the data</b>. You may use the
                    provided <i>Class</i> attribute to identify the currect set of
                    curves. The correct set of curves will assign each entry with
                    a <b>selection probability between 25% and 75% to the class C1</b>.
                    <br />
                    Rate your confidence of being able to correctly identify the
                    correct probability curves.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right, once you feel
                    that you have fulfilled the task.
                </DialogContentText>
            </>);
    }];

    const visible = ['a1', 'a2', 'class'];
    const included = [];
    const { state: initialState, sampleIndices } = validationDataset(visible, included, 500);
    initialState.interactionMode = interactionMode;
    initialState.labels = { 'Default': {} };
    initialState.activeLabel = 'Default';
    initialState.colors = {
        selected: { scale: 'magma', color: 0.5 }
    };
    initialState.colorBar = 'visible';
    initialState.powerProfile = 'high';

    const taskResult = {
        sampleIndices,
        correctCurves: undefined,
        confidence: undefined,
    };

    const taskResultInput = (props: { task: StudyTask, forceUpdate: () => void }): React.JSX.Element => {
        const { task, forceUpdate } = props;
        const { taskResult } = task;
        const { correctCurves, confidence } = taskResult;

        const updateCorrectCurves = (e, value) => {
            taskResult.correctCurves = value ? value : undefined;
            forceUpdate();
        }

        const updateOverallConfidence = (e, value) => {
            taskResult.confidence = value ? value : undefined;
            forceUpdate();
        };

        return (
            <>
                <Typography variant='subtitle1' marginY={2}>
                    Select the correct curves:
                </Typography>
                <Container>
                    <RadioGroup
                        row
                        name='curve-selection'
                        value={correctCurves}
                        onChange={updateCorrectCurves}
                    >
                        <FormControlLabel value='a' control={<Radio />} label='A' />
                        <FormControlLabel value='b' control={<Radio />} label='B' />
                    </RadioGroup>
                </Container>
                <Typography variant='subtitle1' marginY={2}>
                    Rate your confidence:
                </Typography>
                <Container>
                    <Rating
                        name='confidence'
                        value={confidence}
                        max={6}
                        size='large'
                        onChange={updateOverallConfidence}
                        emptyIcon={<StarIcon style={{ opacity: 0.55 }} fontSize='inherit' />}
                    />
                </Container>
            </>);
    };

    const checkCompleted = () => {
        if (taskResult.correctCurves == undefined) {
            return false;
        }

        if (taskResult.confidence === undefined) {
            return false;
        }

        return true;
    }

    return {
        name: 'Select the right curves.',
        shortDescription: 'Select which curves fit the data best.',
        instructions: buildInstructions,
        viewed: false,
        initialState,
        finalState: null,
        taskResult,
        taskResultInput,
        canContinue: (ppc: Props) => checkCompleted()
    };
}

const taskDerivation = (userGroup: UserGroup): StudyTask => {
    const interactionMode = userGroup === 'PC'
        ? InteractionMode.Compatibility
        : InteractionMode.Full;

    const buildInstructions = [() => {
        return (
            <>
                <DialogContentText>
                    For this task, you will look into the Iris dataset. The dataset
                    measures the lengths and widths of the sepals and petals of different
                    iris flowers, which are the <i>iris virginica</i>, <i>iris versicolour</i>
                    &#32;and <i>iris setosa</i>.
                </DialogContentText>
            </>);
    },
    () => {
        return (
            <>
                <DialogContentText>
                    <b>Task:</b><br />
                    Given the provided information, <b>select the entries that are
                        classified as <i>Iris Versicolour</i></b>.
                    For the selection you must try to maximize the number of entries
                    that are truly classified as <i>Iris Versicolour</i>, while minimizing
                    the number of entries belonging to other iris types. To fulfill
                    the task, you may not have any brush applied to the <i>target</i>
                    &#32;attribute, but may utilize it otherwise.
                    <br />
                    Rate your confidence of being able to correctly select the requested
                    entries.
                    <br />
                    <br />
                    Press the <b>Next</b> button on the bottom right, once you feel
                    that you have fulfilled the task.
                </DialogContentText>
            </>);
    }];

    const visible = ['sepal_length', 'sepal_width', 'petal_length', 'petal_width', 'target'];
    const included = [];
    const { state: initialState, sampleIndices } = derivationDataset(visible, included, 500);
    initialState.interactionMode = interactionMode;
    initialState.labels = { 'Default': {} };
    initialState.activeLabel = 'Default';
    initialState.colors = {
        selected: { scale: 'magma', color: 0.5 }
    };
    initialState.colorBar = 'visible';
    initialState.powerProfile = 'high';

    const taskResult = {
        sampleIndices,
        confidence: undefined,
    };

    const taskResultInput = (props: { task: StudyTask, forceUpdate: () => void }): React.JSX.Element => {
        const { task, forceUpdate } = props;
        const { taskResult } = task;
        const { confidence } = taskResult;

        const updateOverallConfidence = (e, value) => {
            taskResult.confidence = value ? value : undefined;
            forceUpdate();
        };

        return (
            <>
                <Typography variant='subtitle1' marginY={2}>
                    Rate your confidence:
                </Typography>
                <Container>
                    <Rating
                        name='confidence'
                        value={confidence}
                        max={6}
                        size='large'
                        onChange={updateOverallConfidence}
                        emptyIcon={<StarIcon style={{ opacity: 0.55 }} fontSize='inherit' />}
                    />
                </Container>
            </>);
    };

    const checkCompleted = (brushes?: { [id: string]: Brushes }) => {
        if (!brushes) {
            return false;
        }

        if (taskResult.confidence === undefined) {
            return false;
        }

        let hasBrushed = false;
        for (const [_, labelBrushes] of Object.entries(brushes)) {
            if ('target' in labelBrushes) {
                return false;
            }
            hasBrushed = hasBrushed || Object.keys(labelBrushes).length != 0;
        }

        return hasBrushed;
    }

    return {
        name: 'Select the Iris Versicolour.',
        shortDescription: 'Select the entries that are classified as "Iris Versicolour".',
        instructions: buildInstructions,
        viewed: false,
        initialState,
        finalState: null,
        taskResult,
        taskResultInput,
        canContinue: (ppc: Props) => checkCompleted(ppc.brushes)
    };
}
