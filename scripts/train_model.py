#!/usr/bin/env python3
"""Train a small vegetation classifier and export to ONNX.

The model is a tiny MLP (3 -> 32 -> 16 -> 1) that classifies individual
RGB pixels as vegetation or background.  It trains on synthetic data by
default but can optionally load labelled images from a directory.

Usage
-----
    # Quick start with synthetic data (no GPU needed):
    pip install torch numpy onnx
    python scripts/train_model.py

    # With a custom dataset directory:
    python scripts/train_model.py --data-dir path/to/images

    # Tune training:
    python scripts/train_model.py --epochs 100 --lr 0.0005 --threshold 0.5

Output
------
    vegetation.onnx   - ONNX model ready for Rust-Spray inference
"""

import argparse
import pathlib
import sys

import numpy as np

try:
    import torch
    import torch.nn as nn
    import torch.optim as optim
except ImportError:
    sys.exit("PyTorch is required.  Install with:  pip install torch")


# ---------------------------------------------------------------------------
# Model
# ---------------------------------------------------------------------------

class VegetationNet(nn.Module):
    """Tiny per-pixel MLP: RGB -> vegetation probability."""

    def __init__(self):
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(3, 32),
            nn.ReLU(),
            nn.Linear(32, 16),
            nn.ReLU(),
            nn.Linear(16, 1),
            nn.Sigmoid(),
        )

    def forward(self, x):
        return self.net(x)


# ---------------------------------------------------------------------------
# Synthetic data generation
# ---------------------------------------------------------------------------

def generate_synthetic(n_samples: int = 20_000):
    """Generate labelled RGB pixels for vegetation / non-vegetation."""
    rng = np.random.default_rng(42)
    half = n_samples // 2

    # --- vegetation ---
    r = rng.uniform(0.05, 0.45, half).astype(np.float32)
    g = rng.uniform(0.45, 1.00, half).astype(np.float32)
    b = rng.uniform(0.02, 0.40, half).astype(np.float32)
    veg = np.stack([r, g, b], axis=1)
    veg_labels = np.ones(half, dtype=np.float32)

    # --- non-vegetation (soil, sky, grey) ---
    kinds = rng.choice(3, half)
    nr = np.empty(half, dtype=np.float32)
    ng = np.empty(half, dtype=np.float32)
    nb = np.empty(half, dtype=np.float32)

    soil = kinds == 0
    nr[soil] = rng.uniform(0.30, 0.70, soil.sum())
    ng[soil] = rng.uniform(0.20, 0.50, soil.sum())
    nb[soil] = rng.uniform(0.10, 0.40, soil.sum())

    sky = kinds == 1
    nr[sky] = rng.uniform(0.40, 0.80, sky.sum())
    ng[sky] = rng.uniform(0.50, 0.90, sky.sum())
    nb[sky] = rng.uniform(0.70, 1.00, sky.sum())

    grey = kinds == 2
    v = rng.uniform(0.20, 0.80, grey.sum())
    nr[grey] = v + rng.uniform(-0.05, 0.05, grey.sum())
    ng[grey] = v + rng.uniform(-0.05, 0.05, grey.sum())
    nb[grey] = v + rng.uniform(-0.05, 0.05, grey.sum())

    non_veg = np.stack([nr, ng, nb], axis=1)
    non_veg_labels = np.zeros(half, dtype=np.float32)

    X = np.concatenate([veg, non_veg])
    y = np.concatenate([veg_labels, non_veg_labels])
    perm = rng.permutation(len(X))
    return X[perm], y[perm]


# ---------------------------------------------------------------------------
# Optional: load labelled images
# ---------------------------------------------------------------------------

def load_image_dataset(data_dir: str):
    """Load labelled images from *data_dir*/vegetation and *data_dir*/background.

    Each subdirectory should contain PNG or JPG images.  Every pixel in a
    vegetation image is labelled positive; every pixel in a background image
    is labelled negative.  This is coarse but sufficient for a small MLP.
    """
    try:
        from PIL import Image
    except ImportError:
        sys.exit("Pillow required for image loading.  pip install Pillow")

    data, labels = [], []
    root = pathlib.Path(data_dir)
    for label_val, subdir in [(1.0, "vegetation"), (0.0, "background")]:
        folder = root / subdir
        if not folder.is_dir():
            print(f"  Warning: {folder} not found, skipping")
            continue
        for img_path in sorted(folder.iterdir()):
            if img_path.suffix.lower() not in (".png", ".jpg", ".jpeg"):
                continue
            img = Image.open(img_path).convert("RGB")
            arr = np.asarray(img, dtype=np.float32).reshape(-1, 3) / 255.0
            data.append(arr)
            labels.append(np.full(len(arr), label_val, dtype=np.float32))
    if not data:
        sys.exit(f"No images found under {root}/vegetation or {root}/background")
    return np.concatenate(data), np.concatenate(labels)


# ---------------------------------------------------------------------------
# Training
# ---------------------------------------------------------------------------

def train(args):
    if args.data_dir:
        print(f"Loading images from {args.data_dir} ...")
        X, y = load_image_dataset(args.data_dir)
    else:
        print("Generating synthetic training data ...")
        X, y = generate_synthetic(args.n_samples)

    print(f"  {len(X)} pixels  ({y.sum():.0f} vegetation, "
          f"{len(y) - y.sum():.0f} background)")

    X_t = torch.from_numpy(X)
    y_t = torch.from_numpy(y).unsqueeze(1)

    model = VegetationNet()
    optimizer = optim.Adam(model.parameters(), lr=args.lr)
    criterion = nn.BCELoss()

    batch_size = 512
    for epoch in range(1, args.epochs + 1):
        perm = torch.randperm(len(X_t))
        total_loss = 0.0
        n_batches = 0
        for i in range(0, len(X_t), batch_size):
            idx = perm[i : i + batch_size]
            pred = model(X_t[idx])
            loss = criterion(pred, y_t[idx])
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
            total_loss += loss.item()
            n_batches += 1

        if epoch % 10 == 0 or epoch == 1:
            avg = total_loss / n_batches
            print(f"  epoch {epoch:4d}/{args.epochs}  loss={avg:.4f}")

    # --- quick accuracy check ---
    with torch.no_grad():
        preds = (model(X_t) > args.threshold).float()
        acc = (preds.squeeze() == y_t.squeeze()).float().mean().item()
        print(f"  training accuracy: {acc:.2%}")

    # --- export to ONNX ---
    dummy = torch.randn(1, 3)
    torch.onnx.export(
        model,
        dummy,
        args.output,
        input_names=["rgb"],
        output_names=["vegetation_prob"],
        dynamic_axes={"rgb": {0: "batch"}, "vegetation_prob": {0: "batch"}},
        opset_version=13,
    )
    print(f"\nModel saved to {args.output}")
    print(f"Use in Rust-Spray:")
    print(f"  cargo build --release --features model")
    print(f'  ModelDetector::load("{args.output}", 640, 480, {args.threshold})')


def main():
    parser = argparse.ArgumentParser(
        description="Train vegetation classifier for Rust-Spray")
    parser.add_argument("--data-dir", default=None,
                        help="Directory with vegetation/ and background/ subfolders")
    parser.add_argument("--n-samples", type=int, default=20_000,
                        help="Synthetic samples to generate (default: 20000)")
    parser.add_argument("--epochs", type=int, default=50,
                        help="Training epochs (default: 50)")
    parser.add_argument("--lr", type=float, default=0.001,
                        help="Learning rate (default: 0.001)")
    parser.add_argument("--threshold", type=float, default=0.5,
                        help="Classification threshold (default: 0.5)")
    parser.add_argument("--output", default="vegetation.onnx",
                        help="Output ONNX path (default: vegetation.onnx)")
    train(parser.parse_args())


if __name__ == "__main__":
    main()
