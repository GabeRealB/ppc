import React, { useEffect, useRef } from 'react';
import _ from 'lodash';

import easingLinearSelRes from '../resources/easing_linear_selected.png'
import easingLinearUnRes from '../resources/easing_linear_unselected.png'

import easingInSelRes from '../resources/easing_in_selected.png'
import easingInUnRes from '../resources/easing_in_unselected.png'

import easingOutSelRes from '../resources/easing_out_selected.png'
import easingOutUnRes from '../resources/easing_out_unselected.png'

import easingInOutSelRes from '../resources/easing_inout_selected.png'
import easingInOutUnRes from '../resources/easing_inout_unselected.png'

import styles from '../../css/PPC.module.css'

import {
    ColorSpace,
    ColorScale,
    SelectedColor,
    Color,
    Colors,
    Axis,
    EasingType,
    LabelInfo,
    Brush,
    Brushes,
    DebugOptions,
    Props,
    InteractionMode
} from '../types'


enum MessageKind {
    StartTransaction,
    EndTransaction,
    Shutdown,
    SetAxes,
    SetAxesOrder,
    SetColors,
    SetColorBarVisibility,
    SetLabels,
    SetBrushes,
    SetInteractionMode,
    SetDebugOptions,
}

type SetAxesMsgPayload = {
    axes: { [id: string]: Axis },
    previousAxes: { [id: string]: Axis },
};

type SetAxesOrderMsgPayload = {
    order: string[],
};

type SetColorsMsgPayload = {
    colors?: Colors
}

type SetColorBarVisibilityMsgPayload = {
    colorBar?: 'hidden' | 'visible',
}

type SetLabelsMsgPayload = {
    labels: { [id: string]: LabelInfo }
    activeLabel: string,
    previousLabels?: { [id: string]: LabelInfo }
    previousActiveLabel?: string,
}

type SetBrushesMsgPayload = { [id: string]: Brushes }

type SetInteractionModeMsgPayload = InteractionMode;
type SetDebugOptionsPayload = DebugOptions;

interface Message {
    kind: MessageKind,
    payload: any
}

/**
 * Component description
 */
const PPC = (props: Props) => {
    const { id, powerProfile } = props;
    const canvasGPURef = useRef<HTMLCanvasElement>(null);
    const canvas2DRef = useRef<HTMLCanvasElement>(null);

    // Create a channel to asynchronously communicate with the js event loop.
    const channelRef = useRef<MessageChannel>(new MessageChannel());
    const sx = channelRef.current.port1;
    const rx = channelRef.current.port2;

    useEffect(() => {
        async function eventLoop() {
            const {
                Renderer,
                PowerProfile,
                AxisDef,
                AxisTicksDef,
                Element,
                ColorDescription,
                ColorScaleDescription,
                DebugOptions,
                StateTransactionBuilder,
            } = await (await import('../../../pkg')).default;

            const canvasGPU = canvasGPURef.current;
            const canvas2D = canvas2DRef.current;

            if (!canvasGPU || !canvas2D) {
                return;
            }

            const callback = (events) => {
                rx.postMessage({ events });
            };

            let profile = PowerProfile.Auto;
            switch (powerProfile) {
                case 'low':
                    profile = PowerProfile.Low;
                    break;
                case 'high':
                    profile = PowerProfile.High;
                    break;
                case 'auto':
                default:
                    profile = PowerProfile.Auto;
            }

            const renderer = await new Renderer(callback, canvasGPU, canvas2D, profile);
            const queue = renderer.constructEventQueue();

            let rendererState = {
                exited: false,
                renderer,
                queue,
            };

            // Listen for Changes in the device pixel ratio.
            const mql = window.matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`);
            mql.addEventListener('change', () => {
                if (rendererState.exited) {
                    return;
                }
                queue.resize(canvas2D.clientWidth, canvas2D.clientHeight, window.devicePixelRatio);
            });

            // Listen for resize events.
            const resizeObserver = new ResizeObserver(() => {
                if (rendererState.exited) {
                    return;
                }
                queue.resize(canvas2D.clientWidth, canvas2D.clientHeight, window.devicePixelRatio);
            });
            resizeObserver.observe(canvas2D);

            // Listen for mouse events.
            canvas2D.addEventListener('pointerdown', (event) => {
                if (rendererState.exited) {
                    return;
                }
                queue.pointerDown(event);
            });
            canvas2D.addEventListener('pointerup', (event) => {
                if (rendererState.exited) {
                    return;
                }
                queue.pointerUp(event);
            });
            canvas2D.addEventListener('pointerleave', (event) => {
                if (rendererState.exited) {
                    return;
                }
                queue.pointerUp(event);
            });
            canvas2D.addEventListener('pointermove', (event) => {
                if (rendererState.exited) {
                    return;
                }
                queue.pointerMove(event);
            });
            canvas2D.addEventListener('contextmenu', (event) => {
                event.preventDefault();
            })

            // Listen for custom events.
            let currentTransaction = new StateTransactionBuilder();
            const shutdown = () => {
                if (!rendererState.exited && rendererState.queue && rendererState.renderer) {
                    rendererState.queue.exit();
                }
                rendererState.exited = true;
            };

            const startTransaction = () => {
                if (currentTransaction) {
                    currentTransaction.free();
                }

                if (rendererState.exited) {
                    return;
                }
                currentTransaction = new StateTransactionBuilder();
            }
            const endTransaction = () => {
                const transaction = currentTransaction.build();
                currentTransaction = undefined;
                rendererState.queue.commitTransaction(transaction);
            }
            const setAxes = (data: SetAxesMsgPayload) => {
                if (rendererState.exited) {
                    return;
                }

                const { axes, previousAxes } = data;
                if (axes) {
                    if (previousAxes) {
                        for (const axis of Object.keys(previousAxes)) {
                            if (!(axis in axes)) {
                                currentTransaction.removeAxis(axis);
                            }
                        }
                    }

                    for (const [id, axis] of Object.entries(axes)) {
                        if (previousAxes) {
                            if (id in previousAxes) {
                                const previousAxis = previousAxes[id];
                                if (_.isEqual(axis, previousAxis)) {
                                    continue;
                                } else {
                                    currentTransaction.removeAxis(id);
                                }
                            }
                        }

                        let hasValidTicks = axis.tickPositions !== undefined;
                        if (hasValidTicks && axis.tickLabels !== undefined) {
                            if (axis.tickPositions.length !== axis.tickLabels.length) {
                                console.warn('Axis has defined tick labels, but the number of tick ' +
                                    'labels does not match the specified tick positions.');
                                hasValidTicks = false;
                            }
                        }

                        const dataPoints = new Float32Array(axis.dataPoints);
                        const range = axis.range ? new Float32Array(axis.range) : undefined;
                        const visibleRange = axis.visibleRange ? new Float32Array(axis.visibleRange) : undefined;
                        const ticks = hasValidTicks ? new AxisTicksDef() : undefined;

                        if (ticks) {
                            for (const position of axis.tickPositions) {
                                ticks.addTick(position);
                            }
                            if (axis.tickLabels) {
                                for (const label of axis.tickLabels) {
                                    ticks.addTickLabel(label);
                                }
                            }
                        }

                        const ax = new AxisDef(id, axis.label, dataPoints, range, visibleRange, ticks);
                        currentTransaction.addAxis(ax);
                    }
                } else {
                    if (previousAxes) {
                        for (const axis of Object.keys(previousAxes)) {
                            currentTransaction.removeAxis(axis);
                        }
                    }
                }
            }
            const setAxesOrder = (data: SetAxesOrderMsgPayload) => {
                if (rendererState.exited) {
                    return;
                }

                const { order } = data;
                currentTransaction.setAxisOrder(order);
            }
            const setColors = (data: SetColorsMsgPayload) => {
                const colors = data.colors;

                if (rendererState.exited) {
                    return;
                }

                if (!colors) {
                    currentTransaction.setDefaultColor(Element.Background);
                    currentTransaction.setDefaultColor(Element.Brush);
                    currentTransaction.setDefaultColor(Element.Unselected);
                    currentTransaction.setDefaultColorScaleColor();
                    currentTransaction.setDefaultSelectedDataColorMode();
                    return;
                }

                const setColor = (element: number, color?: any) => {
                    if (!color) {
                        currentTransaction.setDefaultColor(element);
                        return;
                    }

                    if (color instanceof String) {
                        currentTransaction.setColorNamed(element, color.toString());
                    } else if (typeof color === 'string') {
                        currentTransaction.setColorNamed(element, color);
                    } else {
                        const c = new ColorDescription(color.colorSpace, new Float32Array(color.values));
                        currentTransaction.setColorValue(element, c);
                    }
                }

                const setSelected = (colors?: SelectedColor) => {
                    if (!colors) {
                        currentTransaction.setDefaultColorScaleColor();
                        currentTransaction.setDefaultSelectedDataColorMode();
                        return;
                    }

                    if (!colors.scale) {
                        currentTransaction.setDefaultColorScaleColor();
                    } else {
                        if (colors.scale instanceof String) {
                            currentTransaction.setColorScaleNamed(colors.scale.toString());
                        } else if (typeof colors.scale === 'string') {
                            currentTransaction.setColorScaleNamed(colors.scale);
                        } else if ('values' in colors.scale) {
                            const color: Color = colors.scale;
                            const c = new ColorDescription(color.colorSpace, new Float32Array(color.values));
                            currentTransaction.setColorScaleConstant(c);
                        } else if ('gradient' in colors.scale) {
                            const scale: ColorScale = colors.scale;
                            const s = new ColorScaleDescription(scale.colorSpace);
                            for (let [color, sample] of scale.gradient) {
                                const c = new ColorDescription(color.colorSpace, new Float32Array(color.values));
                                s.withSample(sample, c);
                            }
                            currentTransaction.setColorScaleGradient(s);
                        }
                    }

                    if (colors.color === undefined || colors.color === null) {
                        currentTransaction.setDefaultSelectedDataColorMode();
                    } else {
                        if (colors.color instanceof String) {
                            currentTransaction.setSelectedDataColorModeAttribute(colors.color.toString());
                        } else if (typeof colors.color === 'string') {
                            currentTransaction.setSelectedDataColorModeAttribute(colors.color);
                        } else if (typeof colors.color === 'number') {
                            currentTransaction.setSelectedDataColorModeConstant(colors.color);
                        } else if ('type' in colors.color && colors.color.type === 'probability') {
                            currentTransaction.setSelectedDataColorModeProbability();
                        } else {
                            throw new Error('Unknown color scale color provided');
                        }
                    }
                }

                setColor(Element.Background, colors.background);
                setColor(Element.Brush, colors.brush);
                setColor(Element.Unselected, colors.unselected);
                setSelected(colors.selected);
            }
            const setColorBarVisibility = (data: SetColorBarVisibilityMsgPayload) => {
                if (rendererState.exited) {
                    return;
                }

                let visibility = data.colorBar;
                if (!visibility || visibility === 'hidden') {
                    currentTransaction.setColorBarVisibility(false);
                } else if (visibility === 'visible') {
                    currentTransaction.setColorBarVisibility(true);
                } else {
                    throw new Error('Unknown color bar visibility string')
                }
            }
            const setLabels = (data: SetLabelsMsgPayload) => {
                if (rendererState.exited) {
                    return;
                }

                let labels = data.labels;
                let previousLabels = data.previousLabels ? data.previousLabels : {};

                // Remove old labels.
                for (let id in previousLabels) {
                    if (id in labels === false) {
                        currentTransaction.removeLabel(id);
                    }
                }

                // Update existing labels and add new ones.
                for (let id in labels) {
                    const label = labels[id];
                    if (id in previousLabels === true) {
                        const previous = previousLabels[id];

                        if (label.color !== previous.color) {
                            const color = label.color;
                            const c = new ColorDescription(color.colorSpace, new Float32Array(color.values));
                            currentTransaction.setLabelColor(id, c);
                        }

                        if (label.selectionBounds !== previous.selectionBounds) {
                            const hasSelectionBounds = label.selectionBounds !== undefined;
                            const selectionBoundsStart = hasSelectionBounds ? label.selectionBounds[0] : 0.0;
                            const selectionBoundsEnd = hasSelectionBounds ? label.selectionBounds[1] : 1.0;
                            currentTransaction.setLabelSelectionBounds(id, selectionBoundsStart, selectionBoundsEnd);
                        }

                        if (label.easing !== previous.easing) {
                            currentTransaction.setLabelEasing(id, label.easing);
                        }
                    } else {
                        const color = label.color ? new ColorDescription(label.color.colorSpace, new Float32Array(label.color.values)) : null;
                        const hasSelectionBounds = label.selectionBounds !== undefined;
                        const selectionBoundsStart = hasSelectionBounds ? label.selectionBounds[0] : -1.0;
                        const selectionBoundsEnd = hasSelectionBounds ? label.selectionBounds[1] : -1.0;
                        const easing = label.easing;
                        currentTransaction.addLabel(id, color, hasSelectionBounds, selectionBoundsStart,
                            selectionBoundsEnd, easing);
                    }
                }

                currentTransaction.switchActiveLabel(data.activeLabel);
            };
            const setBrushes = (data: SetBrushesMsgPayload) => {
                if (rendererState.exited) {
                    return;
                }
                currentTransaction.setBrushes(data);
            }
            const setInteractionMode = (mode: SetInteractionModeMsgPayload) => {
                if (rendererState.exited) {
                    return;
                }
                currentTransaction.setInteractionMode(mode);
            }
            const setDebugOptions = (data?: SetDebugOptionsPayload) => {
                if (rendererState.exited) {
                    return;
                }

                const options = new DebugOptions();
                if (data) {
                    options.showAxisBoundingBox = data.showAxisBoundingBox === true;
                    options.showLabelBoundingBox = data.showLabelBoundingBox === true;
                    options.showCurvesBoundingBox = data.showCurvesBoundingBox === true;
                    options.showAxisLineBoundingBox = data.showAxisLineBoundingBox === true;
                    options.showSelectionsBoundingBox = data.showSelectionsBoundingBox === true;
                    options.showColorBarBoundingBox = data.showColorBarBoundingBox === true;
                }
                currentTransaction.setDebugOptions(options);
            }
            const messageListener = (e) => {
                const data: Message = e.data;

                switch (data.kind) {
                    case MessageKind.StartTransaction:
                        startTransaction();
                        break;
                    case MessageKind.EndTransaction:
                        endTransaction();
                        break;
                    case MessageKind.Shutdown:
                        shutdown();
                        break;
                    case MessageKind.SetAxes:
                        setAxes(data.payload);
                        break;
                    case MessageKind.SetAxesOrder:
                        setAxesOrder(data.payload);
                        break;
                    case MessageKind.SetColors:
                        setColors(data.payload);
                        break;
                    case MessageKind.SetColorBarVisibility:
                        setColorBarVisibility(data.payload);
                        break;
                    case MessageKind.SetLabels:
                        setLabels(data.payload);
                        break;
                    case MessageKind.SetBrushes:
                        setBrushes(data.payload);
                        break;
                    case MessageKind.SetInteractionMode:
                        setInteractionMode(data.payload);
                        break;
                    case MessageKind.SetDebugOptions:
                        setDebugOptions(data.payload);
                        break;
                    default:
                        console.warn('unknown message', data);
                }
            };

            // Drawing
            let lastFrameTime = performance.now();
            const fpsInterval = 1000 / 120;
            const draw = async () => {
                if (rendererState.exited) {
                    return;
                }

                const now = performance.now();
                const elapsed = now - lastFrameTime;

                if (elapsed >= fpsInterval) {
                    lastFrameTime = now - (elapsed % fpsInterval);
                    await queue.draw();
                }

                window.requestAnimationFrame(draw);
            };

            rendererState.renderer = renderer;
            rendererState.queue = queue;
            rx.onmessage = messageListener;

            // Start the event loop.
            if (!rendererState.exited) {
                window.requestAnimationFrame(draw);
                await renderer.enterEventLoop();
            }

            // Cleanup.
            queue.free();
            renderer.free();
            rendererState.exited = true;
        }
        eventLoop()

        return () => {
            sx.postMessage({
                kind: MessageKind.Shutdown
            });
        }
    }, [])

    /////////////////////////////////////////////////////
    /// Events
    /////////////////////////////////////////////////////

    // Transaction start
    useEffect(() => {
        sx.postMessage({ kind: MessageKind.StartTransaction, payload: undefined });
    }, [props])

    // Axes update
    const previousAxes = useRef<{ [id: string]: Axis }>(undefined);
    useEffect(() => {
        sx.postMessage({
            kind: MessageKind.SetAxes, payload: {
                axes: props.axes,
                previousAxes: previousAxes.current,
            } as SetAxesMsgPayload
        });
        previousAxes.current = props.axes;
    }, [props.axes]);

    // Order update
    useEffect(() => {
        sx.postMessage({
            kind: MessageKind.SetAxesOrder, payload: {
                order: props.order,
            } as SetAxesOrderMsgPayload
        });
    }, [props.order]);

    // Colors update
    useEffect(() => {
        sx.postMessage({
            kind: MessageKind.SetColors, payload: {
                colors: props.colors
            }
        });
    }, [props.colors]);

    // Color bar update
    useEffect(() => {
        sx.postMessage({
            kind: MessageKind.SetColorBarVisibility, payload: {
                colorBar: props.colorBar
            }
        });
    }, [props.colorBar]);

    // Labels update
    const previousLabels = useRef<{ [id: string]: LabelInfo }>(null);
    const previousActiveLabel = useRef<string>(null);
    useEffect(() => {
        sx.postMessage({
            kind: MessageKind.SetLabels, payload: {
                labels: props.labels,
                activeLabel: props.activeLabel,
                previousLabels: previousLabels.current,
                previousActiveLabel: previousActiveLabel.current,
            }
        });

        previousLabels.current = props.labels;
        previousActiveLabel.current = props.activeLabel;
    }, [props.labels, props.activeLabel]);

    // Brushes update
    useEffect(() => {
        sx.postMessage({ kind: MessageKind.SetBrushes, payload: props.brushes });
    }, [props.brushes])

    // Interaction mode
    useEffect(() => {
        sx.postMessage({ kind: MessageKind.SetInteractionMode, payload: props.interactionMode });
    }, [props.interactionMode]);

    // Debug options
    useEffect(() => {
        sx.postMessage({ kind: MessageKind.SetDebugOptions, payload: props.debug });
    }, [props.debug]);

    // Transaction end
    useEffect(() => {
        sx.postMessage({ kind: MessageKind.EndTransaction, payload: undefined });
    }, [props])

    // Callback handling
    const lastProps = useRef<Props>(props);
    lastProps.current = props;
    const handleAxisOrderChangeEvent = (diff, order) => {
        if (lastProps.current.order) {
            if (_.isEqual(lastProps.current.order, order)) {
                return;
            }
        }

        diff['order'] = order;
    }
    const handleBrushesChangeEvent = (diff, brushes) => {
        if (lastProps.current.brushes) {
            if (_.isEqual(lastProps.current.brushes, brushes)) {
                return;
            }
        }

        diff['brushes'] = brushes;
    }
    const handleProbabilitiesChangeEvent = (diff, value) => {
        const { probabilities, indices } = value;
        const removedLabels = new Set(value.removals);

        let selectionProbabilities = {};
        if (props.selectionProbabilities) {
            for (const [label, v] of Object.entries(props.selectionProbabilities)) {
                if (!(removedLabels.has(label) || label in probabilities)) {
                    selectionProbabilities[label] = v;
                }
            }
            for (const [label, v] of Object.entries(probabilities)) {
                selectionProbabilities[label] = v;
            }
        } else {
            selectionProbabilities = probabilities;
        }

        let selectionIndices = {};
        if (props.selectionIndices) {
            for (const [label, v] of Object.entries(props.selectionIndices)) {
                if (!(removedLabels.has(label) || label in indices)) {
                    selectionIndices[label] = v;
                }
            }
            for (const [label, v] of Object.entries(indices)) {
                selectionIndices[label] = v;
            }
        } else {
            selectionIndices = indices;
        }

        if (!props.selectionProbabilities ||
            (props.selectionProbabilities
                && !_.isEqual(props.selectionProbabilities, selectionProbabilities))) {
            diff['selectionProbabilities'] = selectionProbabilities;
        }

        if (!props.selectionIndices ||
            (props.selectionIndices
                && !_.isEqual(props.selectionIndices, selectionIndices))) {
            diff['selectionIndices'] = selectionIndices;
        }
    }

    // Events
    const handleMessage = (msg) => {
        const { events } = msg.data;

        if (!events) {
            return;
        }

        const diff = {};
        for (const { type, value } of events) {
            switch (type) {
                case 'axis_order':
                    handleAxisOrderChangeEvent(diff, value);
                    break;
                case 'brushes':
                    handleBrushesChangeEvent(diff, value);
                    break;
                case 'probabilities':
                    handleProbabilitiesChangeEvent(diff, value);
                    break;
            }
        }

        if (Object.keys(diff).length != 0) {
            props.setProps(diff);
        }
    }
    sx.onmessage = handleMessage;

    // Plot
    const setEasingCallback = (e) => {
        let labels = window.structuredClone(props.labels);
        const label = labels[props.activeLabel];
        label.easing = e.target.value as EasingType;
        props.setProps({ labels });
    };

    let easing: EasingType = undefined;
    if (props.activeLabel) {
        const { labels } = props;
        const label = labels[props.activeLabel];
        easing = label.easing ? label.easing : 'linear';
    }
    const easingLinearRes = easing == 'linear' ? easingLinearSelRes : easingLinearUnRes;
    const easingInRes = easing == 'in' ? easingInSelRes : easingInUnRes;
    const easingOutRes = easing == 'out' ? easingOutSelRes : easingOutUnRes;
    const easingInOutRes = easing == 'inout' ? easingInOutSelRes : easingInOutUnRes;

    return (
        <div id={id} className={styles.plot}>
            <canvas ref={canvasGPURef} className={styles.gpu}></canvas>
            <canvas ref={canvas2DRef} className={styles.non_gpu}></canvas>
            {props.interactionMode == InteractionMode.Full && props.activeLabel ?
                <div className={styles.toolbar}>
                    <input type='image' src={easingLinearRes} className={styles.toolbar_element} value='linear' onClick={setEasingCallback}></input>
                    <input type='image' src={easingInRes} className={styles.toolbar_element} value='in' onClick={setEasingCallback}></input>
                    <input type='image' src={easingOutRes} className={styles.toolbar_element} value='out' onClick={setEasingCallback}></input>
                    <input type='image' src={easingInOutRes} className={styles.toolbar_element} value='inout' onClick={setEasingCallback}></input>
                </div> : null}
        </div>
    )
}

PPC.defaultProps = {
    axes: {},
    order: [],
    colors: null,
    colorBar: 'hidden',
    labels: {},
    activeLabel: null,
    brushes: {},
    interactionMode: InteractionMode.Full,
    selectionProbabilities: {},
    selectionIndices: {},
};

export default PPC;
