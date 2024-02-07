import { createAdultDataset } from "./datasets/adult";
import { Props } from "types";

export function adultDataset(visible: string[], include: string[], samples?: number): Props {
    const included = new Set([...visible, ...include]);
    const dataset = createAdultDataset(Array.from(included), samples);
    return {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
}