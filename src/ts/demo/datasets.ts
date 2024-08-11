import { createSyntheticTestDataset } from './datasets/synthetic_test';
import { createApplicationDataset } from './datasets/application';
import { createApplicationControlDataset } from './datasets/application_control';
import { createValidationDataset } from './datasets/validation';
import { createValidationControlDataset } from './datasets/validation_control';
import { createDerivationDataset } from './datasets/derivation';
import { createDerivationControlDataset } from './datasets/derivation_control';

import { Props } from 'types';

export function syntheticTestDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createSyntheticTestDataset(Array.from(included), samples);
    const state = {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
    return {
        state, sampleIndices
    };
}

export function applicationDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createApplicationDataset(Array.from(included), samples);
    const state = {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
    return {
        state, sampleIndices
    };
}

export function applicationControlDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createApplicationControlDataset(Array.from(included), samples);
    const state = {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
    return {
        state, sampleIndices
    };
}

export function validationDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createValidationDataset(Array.from(included), samples);
    const state = {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
    return {
        state, sampleIndices
    };
}

export function validationControlDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createValidationControlDataset(Array.from(included), samples);
    const state = {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
    return {
        state, sampleIndices
    };
}

export function derivationDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createDerivationDataset(Array.from(included), samples);
    const state = {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
    return {
        state, sampleIndices
    };
}

export function derivationControlDataset(visible: string[], include: string[], samples?: number): { state: Props, sampleIndices: number[] } {
    const included = new Set([...visible, ...include]);
    const { dataset, sampleIndices } = createDerivationControlDataset(Array.from(included), samples);
    const state = {
        axes: dataset,
        order: visible,
        labels: {},
        setProps: undefined
    };
    return {
        state, sampleIndices
    };
}