import { DashComponentProps } from "./props";

export type ColorSpace = "srgb" | "xyz" | "cie_lab" | "cie lch";

export type Color = {
    colorSpace: ColorSpace,
    values: number[]
};

export type ColorScale = {
    colorSpace: ColorSpace,
    gradient: [Color][] | [Color, number][]
}

export interface ColorProbability {
    type: string
}

export type SelectedColor = {
    scale: string | Color | ColorScale,
    color: number | string | ColorProbability
}

export type Colors = {
    background?: string | Color
    brush?: string | Color
    unselected?: string | Color
    selected: SelectedColor
};

export type Axis = {
    label: string,
    dataPoints: number[],
    range?: [number, number],
    visibleRange?: [number, number],
    tickPositions?: number[],
    tickLabels?: string[],
    hidden?: boolean
};

export type EasingType = "linear" | "in" | "out" | "inout";

export type LabelInfo = {
    color?: Color,
    selectionBounds?: [number, number],
    easing?: EasingType,
}

export type DebugOptions = {
    showAxisBoundingBox?: boolean,
    showLabelBoundingBox?: boolean,
    showCurvesBoundingBox?: boolean,
    showAxisLineBoundingBox?: boolean,
    showSelectionsBoundingBox?: boolean,
    showColorBarBoundingBox?: boolean,
}

export type Brush = {
    controlPoints: [number, number][],
    mainSegmentIdx: number,
}

export type Brushes = { [axis: string]: Brush[] }

export enum InteractionMode {
    /**
     * No interaction enabled.
     */
    Disabled = 0,
    /**
     * Only allow interactions compatible with
     * Parallel Coordinates that don't modify
     * the selection probabilities.
     */
    RestrictedCompatibility = 1,
    /**
     * Only allow interactions compatible with
     * Parallel Coordinates.
     */
    Compatibility = 2,
    /**
     * Only allow interactions that don't modify
     * the selection probabilities.
     */
    Restricted = 3,
    /**
     * Enable all interactions.
     */
    Full = 4
}

export type Props = {
    /**
     * Attribute axes.
     */
    axes?: { [id: string]: Axis },
    /**
     * Order of the attribute axes.
     */
    order?: string[],
    /**
     * Color settings.
     */
    colors?: Colors,
    /**
     * Color bar visibility.
     */
    colorBar?: "hidden" | "visible",
    /**
     * Labels of the selections.
     */
    labels: { [id: string]: LabelInfo },
    /**
     * Currently active label.
     */
    activeLabel?: string,
    /**
     * Per-label map of brushes in the plot.
     */
    brushes?: { [id: string]: Brushes }
    /**
     * Interaction mode of the plot.
     */
    interactionMode?: InteractionMode,
    /**
     * Read-only.
     * 
     * Per label array of selection probabilities 
     * of each point.
     */
    selection_probabilities?: { [id: string]: Float32Array },
    /**
     * Read-only.
     * 
     * Per label array of data indices that count as
     * being selected.
     */
    selection_indices?: { [id: string]: BigUint64Array }
    /**
     * Debug options.
     */
    debug?: DebugOptions,
} & DashComponentProps;