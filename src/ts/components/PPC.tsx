import React, { useEffect, useRef } from 'react';
import { DashComponentProps } from '../props';

type ColorSpace = "srgb" | "xyz" | "cie_lab" | "cie lch";

type Color = {
    color_space: ColorSpace,
    values: number[]
};

type ColorScale = {
    color_space: ColorSpace,
    gradient: [number, Color][]
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
    selection_threshold?: number
}

type Props = {
    axes?: { [id: string]: Axis },
    order?: string[],
    colors?: Colors,
    labels: { [id: string]: LabelInfo },
    active_label: string
} & DashComponentProps;

enum MessageKind {
    Shutdown,
    UpdateData,
    SetColors,
    SetLabels,
}

type UpdateDataMsgPayload = {
    axes?: { [id: string]: Axis },
    order?: string[]
};

type SetColorsMsgPayload = {
    colors?: Colors
}

type SetLabelsMsgPayload = {
    labels: { [id: string]: LabelInfo }
    active_label: string,
    previous_labels?: { [id: string]: LabelInfo }
    previous_active_label?: string,
}

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

            const renderer = await new Renderer(canvasGPU, canvas2D);
            const queue = renderer.construct_event_queue();

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
                queue.pointer_down(event);
            });
            canvas2D.addEventListener("pointerup", (event) => {
                if (rendererState.exited) {
                    return;
                }
                queue.pointer_up(event);
            });
            canvas2D.addEventListener("pointerleave", (event) => {
                if (rendererState.exited) {
                    return;
                }
                queue.pointer_up(event);
            });
            canvas2D.addEventListener("pointermove", (event) => {
                if (rendererState.exited) {
                    return;
                }
                queue.pointer_move(event);
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
                        payload.new_axis(key, axis.label, datums, range, visibleRange, axis.hidden);
                    }
                }

                if (order) {
                    for (let key of order) {
                        payload.add_order(key);
                    }
                }

                rendererState.queue.update_data(payload);
            }
            const setColors = (data: SetColorsMsgPayload) => {
                const colors = data.colors;

                if (rendererState.exited) {
                    return;
                }

                if (!colors) {
                    rendererState.queue.set_default_color(Element.Background);
                    rendererState.queue.set_default_color(Element.Brush);
                    rendererState.queue.set_default_color(Element.Unselected);
                    rendererState.queue.set_default_color_scale_color();
                    rendererState.queue.set_default_selected_datum_coloring();
                    return;
                }

                const set_color = (element: number, color?: any) => {
                    if (!color) {
                        rendererState.queue.set_default_color(element);
                        return;
                    }

                    if (color instanceof String) {
                        rendererState.queue.set_color_named(element, color.toString());
                    } else if (typeof color === 'string') {
                        rendererState.queue.set_color_named(element, color);
                    } else {
                        const c = new ColorDescription(color.color_space, new Float32Array(color.values));
                        rendererState.queue.set_color_value(element, c);
                    }
                }

                const set_selected = (colors?: SelectedColor) => {
                    if (!colors) {
                        rendererState.queue.set_default_color_scale_color();
                        rendererState.queue.set_default_selected_datum_coloring();
                        return;
                    }

                    if (!colors.scale) {
                        rendererState.queue.set_default_color_scale_color();
                    } else {
                        if (colors.scale instanceof String) {
                            rendererState.queue.set_color_scale_named(colors.scale.toString());
                        } else if (typeof colors.scale === 'string') {
                            rendererState.queue.set_color_scale_named(colors.scale);
                        } else if ('values' in colors.scale) {
                            const color: Color = colors.scale;
                            const c = new ColorDescription(color.color_space, new Float32Array(color.values));
                            rendererState.queue.set_color_scale_constant(c);
                        } else if ('gradient' in colors.scale) {
                            const scale: ColorScale = colors.scale;
                            const s = new ColorScaleDescription(scale.color_space);
                            for (let [sample, color] of scale.gradient) {
                                const c = new ColorDescription(color.color_space, new Float32Array(color.values));
                                s.with_sample(sample, c);
                            }
                            rendererState.queue.set_color_scale_gradient(s);
                        }
                    }

                    if (!colors.color) {
                        rendererState.queue.set_default_selected_datum_coloring();
                    } else {
                        if (colors.color instanceof String) {
                            rendererState.queue.set_selected_datum_coloring_attribute(colors.color.toString());
                        } else if (typeof colors.color === 'string') {
                            rendererState.queue.set_selected_datum_coloring_attribute(colors.color);
                        } else if (typeof colors.color === 'number') {
                            rendererState.queue.set_selected_datum_coloring_constant(colors.color);
                        } else if ('type' in colors.color && colors.color.type === 'probability') {
                            rendererState.queue.set_selected_datum_coloring_by_probability();
                        } else {
                            throw new Error("Unknown color scale color provided");
                        }
                    }
                }

                set_color(Element.Background, colors.background);
                set_color(Element.Brush, colors.brush);
                set_color(Element.Unselected, colors.unselected);
                set_selected(colors.selected);
            }
            const set_labels = (data: SetLabelsMsgPayload) => {
                if (rendererState.exited) {
                    return;
                }

                let labels = data.labels;
                let previous_labels = data.previous_labels ? data.previous_labels : {};

                // Remove old labels.
                for (let id in previous_labels) {
                    if (id in labels === false) {
                        rendererState.queue.remove_label(id);
                    }
                }

                // Update existing labels and add new ones.
                for (let id in labels) {
                    const label = labels[id];
                    if (id in previous_labels === true) {
                        const previous = previous_labels[id];

                        if (label.color !== previous.color) {
                            const color = label.color;
                            if (color) {
                                const c = new ColorDescription(color.color_space, new Float32Array(color.values));
                                rendererState.queue.set_label_color(id, c);
                            } else {
                                rendererState.queue.set_label_color(id, null);
                            }
                        }

                        if (label.selection_threshold !== previous.selection_threshold) {
                            rendererState.queue.set_label_selection_threshold(id, label.selection_threshold);
                        }
                    } else {
                        let color = label.color ? new ColorDescription(label.color.color_space, new Float32Array(label.color.values)) : null;
                        let selection_threshold = label.selection_threshold;
                        rendererState.queue.add_label(id, color, selection_threshold);
                    }
                }

                if (data.active_label !== data.previous_active_label) {
                    rendererState.queue.switch_active_label(data.active_label);
                }
            };
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
                    case MessageKind.SetLabels:
                        set_labels(data.payload);
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
                await renderer.enter_event_loop();
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

    // Labels update
    const previous_labels = useRef<{ [id: string]: LabelInfo }>(null);
    const previous_active_label = useRef<string>(null);
    useEffect(() => {
        sx.postMessage({
            kind: MessageKind.SetLabels, payload: {
                labels: props.labels,
                active_label: props.active_label,
                previous_labels: previous_labels.current,
                previous_active_label: previous_active_label.current,
            }
        });

        previous_labels.current = props.labels;
        previous_active_label.current = props.active_label;
    }, [props.labels, props.active_label]);

    return (
        <div id={id} style={{ position: "relative", width: "100%", height: "100%" }}>
            <canvas ref={canvasGPURef} style={{ position: "absolute", left: 0, top: 0, zIndex: 0, width: "100%", height: "100%" }}></canvas>
            <canvas ref={canvas2DRef} style={{ position: "absolute", left: 0, top: 0, zIndex: 1, width: "100%", height: "100%" }}></canvas>
        </div>
    )
}

PPC.defaultProps = {
    labels: {
        "unknown": {}
    },
    active_label: "unknown"
};

export default PPC;
