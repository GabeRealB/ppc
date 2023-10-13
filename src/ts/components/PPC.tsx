import React, { useEffect, useRef } from 'react';
import { DashComponentProps } from '../props';

type Props = {
    // Insert props
} & DashComponentProps;

/**
 * Component description
 */
const PPC = (props: Props) => {
    const { id } = props;
    const canvasGPURef = useRef<HTMLCanvasElement>(null);
    const canvas2DRef = useRef<HTMLCanvasElement>(null);

    useEffect(() => {
        let rendererState = {
            exited: false,
            renderer: null,
            queue: null,
        };

        async function eventLoop() {
            const { EventQueue, Renderer } = await (await import('../../../pkg')).default;

            const canvasGPU = canvasGPURef.current;
            const canvas2D = canvas2DRef.current;

            if (!canvasGPU || !canvas2D || rendererState.exited) {
                return;
            }

            const renderer = await new Renderer(canvasGPU, canvas2D);
            const queue = renderer.construct_event_queue();

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

            // Drawing
            let animationFrameId;
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

                animationFrameId = window.requestAnimationFrame(draw);
            };

            rendererState.renderer = renderer;
            rendererState.queue = queue;

            // Start the event loop.
            if (!rendererState.exited) {
                animationFrameId = window.requestAnimationFrame(draw);
                await renderer.enter_event_loop();
            }

            // Cleanup.
            queue.free();
            renderer.free();
            rendererState.exited = true;
        }
        eventLoop();

        return () => {
            if (!rendererState.exited && rendererState.queue && rendererState.renderer) {
                rendererState.queue.exit();
            }
            rendererState.exited = true;
        }
    }, [])

    return (
        <div id={id} style={{ position: "relative", width: "100%", height: "100%" }}>
            <canvas ref={canvasGPURef} style={{ position: "absolute", left: 0, top: 0, zIndex: 0, width: "100%", height: "100%" }}></canvas>
            <canvas ref={canvas2DRef} style={{ position: "absolute", left: 0, top: 0, zIndex: 1, width: "100%", height: "100%" }}></canvas>
        </div>
    )
}

PPC.defaultProps = {};

export default PPC;
