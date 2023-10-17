import React, { useEffect, useRef } from 'react';
import { DashComponentProps } from '../props';

type Axis = {
    label: string,
    datums: number[],
    range?: [number, number],
    visibleRange?: [number, number],
    hidden?: boolean
};

type Props = {
    axes?: { [id: string]: Axis },
    order?: string[]
} & DashComponentProps;

enum MessageKind {
    Shutdown,
    UpdateData
}

type UpdateDataMsgPayload = {
    axes?: { [id: string]: Axis },
    order?: string[]
};

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
    const channel = new MessageChannel();
    const sx = channel.port1;
    const rx = channel.port2;

    useEffect(() => {
        async function eventLoop() {
            const { EventQueue, Renderer, UpdateDataPayload } = await (await import('../../../pkg')).default;

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
            const messageListener = (e) => {
                const data: Message = e.data;

                switch (data.kind) {
                    case MessageKind.Shutdown:
                        shutdown();
                        break;
                    case MessageKind.UpdateData:
                        updateData(data.payload);
                        break;
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

    return (
        <div id={id} style={{ position: "relative", width: "100%", height: "100%" }}>
            <canvas ref={canvasGPURef} style={{ position: "absolute", left: 0, top: 0, zIndex: 0, width: "100%", height: "100%" }}></canvas>
            <canvas ref={canvas2DRef} style={{ position: "absolute", left: 0, top: 0, zIndex: 1, width: "100%", height: "100%" }}></canvas>
        </div>
    )
}

PPC.defaultProps = {};

export default PPC;
