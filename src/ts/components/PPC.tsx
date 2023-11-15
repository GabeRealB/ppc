import React, { useEffect, useRef, useState } from 'react';
import { DashComponentProps } from '../props';

import easingLinearSelRes from '../resources/easing_linear_selected.png'
import easingLinearUnRes from '../resources/easing_linear_unselected.png'

import easingInSelRes from '../resources/easing_in_selected.png'
import easingInUnRes from '../resources/easing_in_unselected.png'

import easingOutSelRes from '../resources/easing_out_selected.png'
import easingOutUnRes from '../resources/easing_out_unselected.png'

import easingInOutSelRes from '../resources/easing_inout_selected.png'
import easingInOutUnRes from '../resources/easing_inout_unselected.png'

import styles from './PPC.module.css'

type ColorSpace = "srgb" | "xyz" | "cie_lab" | "cie lch";

type Color = {
    colorSpace: ColorSpace,
    values: number[]
};

type ColorScale = {
    colorSpace: ColorSpace,
    gradient: [Color][] | [Color, number][]
}

type SelectedColor = {
    scale: string | Color | ColorScale,
    color: number | string | { type: "probability" }
}

type Colors = {
    background?: string | Color
    brush?: string | Color
    unselected?: string | Color
    selected: SelectedColor
};

type Axis = {
    label: string,
    datums: number[],
    range?: [number, number],
    visibleRange?: [number, number],
    hidden?: boolean
};

type LabelInfo = {
    color?: Color,
    selectionThreshold?: number
}

type Props = {
    axes?: { [id: string]: Axis },
    order?: string[],
    colors?: Colors,
    colorBar?: "hidden" | "visible",
    labels: { [id: string]: LabelInfo },
    activeLabel: string
} & DashComponentProps;

enum MessageKind {
    Shutdown,
    UpdateData,
    SetColors,
    SetColorBarVisibility,
    SetLabels,
    SetEasing,
}

type UpdateDataMsgPayload = {
    axes?: { [id: string]: Axis },
    order?: string[]
};

type SetColorsMsgPayload = {
    colors?: Colors
}

type SetColorBarVisibilityMsgPayload = {
    colorBar?: "hidden" | "visible",
}

type SetLabelsMsgPayload = {
    labels: { [id: string]: LabelInfo }
    activeLabel: string,
    previousLabels?: { [id: string]: LabelInfo }
    previousActiveLabel?: string,
}

type SetEasingMsgPayload = string;

interface Message {
    kind: MessageKind,
    payload: any
}

/**
 * Component description
 */
const PPC = (props: Props) => {
    const { id } = props;
    const canvasGPURef = useRef<HTMLCanvasElement>(null);
    const canvas2DRef = useRef<HTMLCanvasElement>(null);

    // Create a channel to asynchronously communicate with the js event loop.
    const channelRef = useRef<MessageChannel>(new MessageChannel());
    const sx = channelRef.current.port1;
    const rx = channelRef.current.port2;

    useEffect(() => {
        async function eventLoop() {
            const { Renderer, UpdateDataPayload, ColorScaleDescription, ColorDescription, Element } = await (await import('../../../pkg')).default;

            const canvasGPU = canvasGPURef.current;
            const canvas2D = canvas2DRef.current;

            if (!canvasGPU || !canvas2D) {
                return;
            }

            const callback = (event, data) => {
                rx.postMessage({ event, data });
            };

            const renderer = await new Renderer(callback, canvasGPU, canvas2D);
            const queue = renderer.constructEventQueue();

            let rendererState = {
                exited: false,
                renderer,
                queue,
            };

            // Listen for Changes in the device pixel ratio.
            const mql = window.matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`);
            mql.addEventListener("change", () => {
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
            canvas2D.addEventListener("pointerdown", (event) => {
                if (rendererState.exited) {
                    return;
                }
                queue.pointerDown(event);
            });
            canvas2D.addEventListener("pointerup", (event) => {
                if (rendererState.exited) {
                    return;
                }
                queue.pointerUp(event);
            });
            canvas2D.addEventListener("pointerleave", (event) => {
                if (rendererState.exited) {
                    return;
                }
                queue.pointerUp(event);
            });
            canvas2D.addEventListener("pointermove", (event) => {
                if (rendererState.exited) {
                    return;
                }
                queue.pointerMove(event);
            });

            // Listen for custom events.
            const shutdown = () => {
                console.log("Shutdown")
                if (!rendererState.exited && rendererState.queue && rendererState.renderer) {
                    rendererState.queue.exit();
                }
                rendererState.exited = true;
            };
            const updateData = (data: UpdateDataMsgPayload) => {
                const axes = data.axes;
                const order = data.order;

                if (rendererState.exited) {
                    return;
                }

                const payload = new UpdateDataPayload();
                if (axes) {
                    for (let key in axes) {
                        const axis = axes[key];

                        const datums = new Float32Array(axis.datums);
                        const range = axis.range ? new Float32Array(axis.range) : undefined;
                        const visibleRange = axis.visibleRange ? new Float32Array(axis.visibleRange) : undefined;
                        payload.newAxis(key, axis.label, datums, range, visibleRange, axis.hidden);
                    }
                }

                if (order) {
                    for (let key of order) {
                        payload.addOrder(key);
                    }
                }

                rendererState.queue.updateData(payload);
            }
            const setColors = (data: SetColorsMsgPayload) => {
                const colors = data.colors;

                if (rendererState.exited) {
                    return;
                }

                if (!colors) {
                    rendererState.queue.setDefaultColor(Element.Background);
                    rendererState.queue.setDefaultColor(Element.Brush);
                    rendererState.queue.setDefaultColor(Element.Unselected);
                    rendererState.queue.setDefaultColorScaleColor();
                    rendererState.queue.setDefaultSelectedDatumColoring();
                    return;
                }

                const setColor = (element: number, color?: any) => {
                    if (!color) {
                        rendererState.queue.setDefaultColor(element);
                        return;
                    }

                    if (color instanceof String) {
                        rendererState.queue.setColorNamed(element, color.toString());
                    } else if (typeof color === 'string') {
                        rendererState.queue.setColorNamed(element, color);
                    } else {
                        const c = new ColorDescription(color.colorSpace, new Float32Array(color.values));
                        rendererState.queue.setColorValue(element, c);
                    }
                }

                const setSelected = (colors?: SelectedColor) => {
                    if (!colors) {
                        rendererState.queue.setDefaultColorScaleColor();
                        rendererState.queue.setDefaultSelectedDatumColoring();
                        return;
                    }

                    if (!colors.scale) {
                        rendererState.queue.setDefaultColorScaleColor();
                    } else {
                        if (colors.scale instanceof String) {
                            rendererState.queue.setColorScaleNamed(colors.scale.toString());
                        } else if (typeof colors.scale === 'string') {
                            rendererState.queue.setColorScaleNamed(colors.scale);
                        } else if ('values' in colors.scale) {
                            const color: Color = colors.scale;
                            const c = new ColorDescription(color.colorSpace, new Float32Array(color.values));
                            rendererState.queue.setColorScaleConstant(c);
                        } else if ('gradient' in colors.scale) {
                            const scale: ColorScale = colors.scale;
                            const s = new ColorScaleDescription(scale.colorSpace);
                            for (let [color, sample] of scale.gradient) {
                                const c = new ColorDescription(color.colorSpace, new Float32Array(color.values));
                                s.withSample(sample, c);
                            }
                            rendererState.queue.setColorScaleGradient(s);
                        }
                    }

                    if (!colors.color) {
                        rendererState.queue.setDefaultSelectedDatumColoring();
                    } else {
                        if (colors.color instanceof String) {
                            rendererState.queue.setSelectedDatumColoringAttribute(colors.color.toString());
                        } else if (typeof colors.color === 'string') {
                            rendererState.queue.setSelectedDatumColoringAttribute(colors.color);
                        } else if (typeof colors.color === 'number') {
                            rendererState.queue.setSelectedDatumColoringConstant(colors.color);
                        } else if ('type' in colors.color && colors.color.type === 'probability') {
                            rendererState.queue.setSelectedDatumColoringByProbability();
                        } else {
                            throw new Error("Unknown color scale color provided");
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
                if (!visibility || visibility === "hidden") {
                    rendererState.queue.setColorBarVisibility(false);
                } else if (visibility === "visible") {
                    rendererState.queue.setColorBarVisibility(true);
                } else {
                    throw new Error("Unknown color bar visibility string")
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
                        rendererState.queue.removeLabel(id);
                    }
                }

                // Update existing labels and add new ones.
                for (let id in labels) {
                    const label = labels[id];
                    if (id in previousLabels === true) {
                        const previous = previousLabels[id];

                        if (label.color !== previous.color) {
                            const color = label.color;
                            if (color) {
                                const c = new ColorDescription(color.colorSpace, new Float32Array(color.values));
                                rendererState.queue.setLabelColor(id, c);
                            } else {
                                rendererState.queue.setLabelColor(id, null);
                            }
                        }

                        if (label.selectionThreshold !== previous.selectionThreshold) {
                            rendererState.queue.setLabelSelectionThreshold(id, label.selectionThreshold);
                        }
                    } else {
                        let color = label.color ? new ColorDescription(label.color.colorSpace, new Float32Array(label.color.values)) : null;
                        let selectionThreshold = label.selectionThreshold;
                        rendererState.queue.addLabel(id, color, selectionThreshold);
                    }
                }

                if (data.activeLabel !== data.previousActiveLabel) {
                    rendererState.queue.switchActiveLabel(data.activeLabel);
                }
            };
            const setEasing = (data: SetEasingMsgPayload) => {
                if (rendererState.exited) {
                    return;
                }
                rendererState.queue.setLabelEasing(data);
            }
            const messageListener = (e) => {
                const data: Message = e.data;

                switch (data.kind) {
                    case MessageKind.Shutdown:
                        shutdown();
                        break;
                    case MessageKind.UpdateData:
                        updateData(data.payload);
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
                    case MessageKind.SetEasing:
                        setEasing(data.payload);
                        break;
                    default:
                        console.log("unknown message", data);
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
        eventLoop();

        return () => {
            sx.postMessage({
                kind: MessageKind.Shutdown
            });
        }
    }, [])

    /////////////////////////////////////////////////////
    /// Events
    /////////////////////////////////////////////////////

    // Data update
    useEffect(() => {
        sx.postMessage({
            kind: MessageKind.UpdateData, payload: {
                axes: props.axes,
                order: props.order,
            }
        });
    }, [props.axes, props.order]);

    // Color update
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

    const [easing, setEasing] = useState<string>("linear");
    const easingLinearRes = easing == "linear" ? easingLinearSelRes : easingLinearUnRes;
    const easingInRes = easing == "in" ? easingInSelRes : easingInUnRes;
    const easingOutRes = easing == "out" ? easingOutSelRes : easingOutUnRes;
    const easingInOutRes = easing == "inout" ? easingInOutSelRes : easingInOutUnRes;
    useEffect(() => {
        sx.postMessage({
            kind: MessageKind.SetEasing, payload: easing
        });
    }, [easing]);

    // Callback handling
    const handleEasingChangeEvent = (easing) => {
        setEasing(easing);
    }

    // Events
    const handleMessage = (msg) => {
        const event = msg.data.event;
        const data = msg.data.data;

        if (event === "easing") {
            handleEasingChangeEvent(data);
        }
    }
    sx.onmessage = handleMessage;

    // Plot
    const setEasingCallback = (e) => {
        setEasing(e.target.value);
    };

    return (
        <div id={id} className={styles.plot}>
            <canvas ref={canvasGPURef} className={styles.gpu}></canvas>
            <canvas ref={canvas2DRef} className={styles.non_gpu}></canvas>
            <div className={styles.toolbar}>
                <input type="image" src={easingLinearRes} className={styles.toolbar_element} value="linear" onClick={setEasingCallback}></input>
                <input type="image" src={easingInRes} className={styles.toolbar_element} value="in" onClick={setEasingCallback}></input>
                <input type="image" src={easingOutRes} className={styles.toolbar_element} value="out" onClick={setEasingCallback}></input>
                <input type="image" src={easingInOutRes} className={styles.toolbar_element} value="inout" onClick={setEasingCallback}></input>
            </div>
        </div>
    )
}

PPC.defaultProps = {
    labels: {
        "unknown": {}
    },
    activeLabel: "unknown"
};

export default PPC;
